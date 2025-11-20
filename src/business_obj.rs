use std::sync::Mutex;

use std::sync::Arc;

/// Starting with a Arc<Mutex<TYPE>>, the BusinessObject trait allows UI applications to pass around refereces without having to manage multiple mutexes. An alernative is to use mapped MutexGuards.
pub(crate) trait BusinessObject: Sized + Clone {
    type Type;

    /// Exeute any locking and dereferecing to get access to the reference.
    fn exec<F, R>(&self, f: F) -> R
    where
        F: FnMut(&mut Self::Type) -> R;

    /// An inner reference to a business object without using fine grained locks.
    fn map<F, X>(self, f: F) -> impl BusinessObject<Type = X>
    where
        F: Fn(&mut Self::Type) -> &mut X,
    {
        Arc::new(BusinessObjectRef {
            base: self,
            function: f,
        })
    }
}

impl<T: BusinessObject, X, FR: Fn(&mut T::Type) -> &mut X> BusinessObject
    for Arc<BusinessObjectRef<T, X, FR>>
{
    type Type = X;

    fn exec<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut Self::Type) -> R,
    {
        self.base.exec(|b| f((self.function)(b)))
    }
}

#[derive(Clone)]
pub(crate) struct BusinessObjectRef<T: BusinessObject, X, F: Fn(&mut T::Type) -> &mut X> {
    pub(crate) base: T,
    pub(crate) function: F,
}

impl<BASE> BusinessObject for Arc<Mutex<BASE>> {
    type Type = BASE;

    fn exec<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut BASE) -> R,
    {
        let mut g = self.lock().unwrap();
        f(&mut *g)
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use std::sync::Mutex;

    use std::sync::Arc;

    use crate::business_obj::BusinessObject;

    #[derive(Debug)]
    pub(crate) struct City {
        pub(crate) name: &'static str,
        pub(crate) state: &'static str,
        pub(crate) zip: u16,
    }

    #[derive(Debug)]
    pub(crate) struct Address {
        pub(crate) street: &'static str,
        pub(crate) city: City,
    }

    #[derive(Debug)]
    pub(crate) struct Person {
        pub(crate) name: &'static str,
        pub(crate) address: Address,
    }

    #[test]
    pub(crate) fn joe() {
        let b1 = Arc::new(Mutex::new(Person {
            name: "joe",
            address: Address {
                street: "1526 S Base",
                city: City {
                    name: "Winchester",
                    state: "IN",
                    zip: 47394,
                },
            },
        }));
        b1.exec(|person| eprintln!("person.name is {}", person.name));

        let address = b1.map(|person| &mut person.address);
        address.exec(|v| eprintln!("address is {v:?}"));

        let city = address.map(|address| &mut address.city);
        city.exec(|v| eprintln!("city is {v:?}"));
    }
}
