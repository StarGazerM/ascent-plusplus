
// phantom vector is special vector that push it is a no-op
// but equipped a push_really method that actually push the element

pub struct PhantomVec<T> {
    data: Vec<T>,
}

impl<T> PhantomVec<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push_really(&mut self, value: T) {
        self.data.push(value);
    }

    pub fn push(&self, _: T) {
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }
}

impl<T> Default for PhantomVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl <T> std::ops::Index<usize> for PhantomVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> FromIterator<T> for PhantomVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Self::new();
        for i in iter {
            vec.push_really(i);
        }
        vec
    }
}
