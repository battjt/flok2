use std::sync::{Arc, Mutex};

/// Starting with a Arc<Mutex<TYPE>>, the BusinessObject trait allows UI applications to pass around refereces without having to manage multiple mutexes. An alernative is to use mapped MutexGuards.
pub trait BusinessObject: Sized + Clone {
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
struct BusinessObjectRef<T: BusinessObject, X, F: Fn(&mut T::Type) -> &mut X> {
    base: T,
    function: F,
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
    use crate::business_obj::BusinessObject;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct City {
        name: &'static str,
        state: &'static str,
        zip: u16,
    }

    #[derive(Debug)]
    struct Address {
        street: &'static str,
        city: City,
    }

    #[derive(Debug)]
    struct Person {
        name: &'static str,
        address: Address,
    }

    #[test]
    fn joe() {
        let person = Arc::new(Mutex::new(Person {
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
        person.exec(|person| eprintln!("person.name is {}", person.name));

        let (city, address) = {
            // test letting the "original" fall out of scope
            let address = person.map(|person| &mut person.address);
            let city = address.clone().map(|address| &mut address.city);

            (city.clone(), address.clone())
        };

        address.exec(|v| eprintln!("address is {v:?}"));
        address.exec(|v| eprintln!("address.city is {:?}", v.city));
        city.exec(|v| eprintln!("city is {v:?}"));
    }
}
