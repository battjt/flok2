use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

///
/// The purposes is to make a business object instance behind a Arc<Mutex<>> easier to access.
///
/// ```
/// let first_child_reference = BusinessObject::new(person).map(|p|p.children[0]);
/// let button = Button::defult();
/// button.set_callback(|_|{
///   first_child_reference.exec(|c|eprintln!("first child's name is {}", c.name));
/// });
/// ```
///
/// TYPE is the type of be base object (AKA Business Object)
/// RESULT is the type of the attribute of the business object that self is exposing.
#[derive(Clone)]
pub struct BusinessObject<'a, TYPE: 'a, RESULT: 'a, F: 'a> {
    p: PhantomData<&'a RESULT>,
    object: Arc<Mutex<TYPE>>,
    function: F,
}
impl<'a, TYPE, RESULT, FN> BusinessObject<'a, TYPE, RESULT, FN>
where
    FN: (Fn(MutexGuard<'a, TYPE>) -> MutexGuard<'a, RESULT>) + Clone,
{
    pub fn new(base: Arc<Mutex<TYPE>>, function: FN) -> Self {
        Self {
            p: PhantomData,
            object: base,
            function,
        }
    }

    /// The meat of the struct. This is how the BusinessObj is used.
    /// ```
    /// Button save_button = todo!();
    /// Input name=todo!();
    /// Input city=todo!();
    /// let p: BusinessObject<Person,Person,_>=todo!();
    /// let city = p.map(|p|&p.city.name);
    /// save_button.set_callback(move |b|{
    ///   p.exec(|p|p.name=value.value());
    ///   city.exec(|c|c.name=city.value());
    /// });
    pub fn exec<E>(&'a self, f: E)
    where
        E: (Fn(MutexGuard<'a, RESULT>) -> ()),
    {
        let object = self.object.lock().unwrap();
        let x = (&self.function)(object);
        f(x);
    }

    /// How to reference something within the BO.  For instance, each edit button in a table may have a different mapped BO.
    pub fn mapbo<G, R: 'a>(
        &self,
        g: G,
    ) -> BusinessObject<'a, TYPE, R, impl (Fn(MutexGuard<'a, TYPE>) -> MutexGuard<'a, R>) + Clone>
    where
        G: (Fn(MutexGuard<'a, RESULT>) -> MutexGuard<R>) + 'a + Clone,
    {
        let f = self.function.clone();
        let composed_fn = move |x: MutexGuard<'a, TYPE>| g(f(x));
        BusinessObject {
            p: PhantomData,
            object: self.object.clone(),
            function: composed_fn,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::reference::BusinessObject;

    struct Person {
        name: &'static str,
        address: Address,
    }
    struct Address {
        local: &'static str,
        city: City,
    }
    struct City {
        name: &'static str,
        zip: u32,
        state: State,
    }
    enum State {
        Indiana,
        Ohio,
        Others,
    }
    #[test]
    fn simple() {
        let joe = Person {
            name: "Joe",
            address: Address {
                local: "123 main",
                city: City {
                    name: "town",
                    zip: 123,
                    state: State::Indiana,
                },
            },
        };

        let function = |x| x;
        let bo = BusinessObject::new(Arc::new(Mutex::new(joe)), function);
        let address_ref = bo.mapbo(|p| p.map(address);
        let state_ref = address_ref.mapbo(|a| a.city.state);

        //        assert_eq!("123 main".to_string(), address_ref.exec(|a| a.local));
        state_ref.exec(|s| assert!(matches!(s, State::Indiana)));
    }
}
