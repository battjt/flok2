use std::{
    cell::Ref,
    clone,
    sync::{Arc, Mutex},
};

pub trait BusObj<TYPE> {
    fn exec<FN, RESULT>(&self, f: FN) -> RESULT
    where
        FN: Fn(&mut TYPE) -> RESULT;

    fn map<FN, RESULT, RBO>(&self, f: FN) -> RBO
    where
        FN: for<'a> Fn(&'a mut TYPE) -> &'a mut RESULT,
        RBO: BusObj<RESULT>,
    {
        Reference::new(self.clone(), f)
    }
}

#[derive(Clone)]
pub struct BusinessObject<T> {
    object: Arc<Mutex<T>>,
}
impl<T> BusinessObject<T> {
    pub fn new(o: T) -> Self {
        BusinessObject {
            object: Arc::new(Mutex::new(o)),
        }
    }
}
impl<TYPE> BusObj<TYPE> for BusinessObject<TYPE> {
    fn exec<FN, RESULT>(&self, mut f: FN) -> RESULT
    where
        FN: Fn(&mut TYPE) -> RESULT,
    {
        let object = &mut *self.object.lock().unwrap();
        f(object)
    }
}

pub struct Reference<BO, RESULT, BOTYPE: BusObj<BO>> {
    object: BOTYPE,
    function: Box<dyn for<'a> Fn(&'a mut BO) -> &'a mut RESULT>,
}

impl<BO, T, BOTYPE: BusObj<BO>> Reference<BO, T, BOTYPE> {
    fn new<FN>(base: BOTYPE, f: FN) -> Self
    where
        FN: for<'a> Fn(&'a mut BO) -> &'a mut T,
    {
        Self {
            object: base,
            function: Box::new(f),
        }
    }
}

impl<O, BOTYPE: BusObj<O>, TYPE> BusObj<TYPE> for Reference<O, TYPE, BOTYPE> {
    fn exec<FN, RESULT>(&self, mut f: FN) -> RESULT
    where
        FN: FnMut(&mut TYPE) -> RESULT,
    {
        self.object.exec(|o| {
            let object = (self.function)(o);
            f(object)
        })
    }
}
