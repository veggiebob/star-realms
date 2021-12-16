use rand::Rng;
use std::slice::Iter;
use std::collections::HashSet;

pub trait Stack {
    type Item;
    fn len(&self) -> usize;
    fn remove(&mut self, index: usize) -> Option<Self::Item>;
    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn add(&mut self, item: Self::Item);
    fn iter(&self) -> Iter<'_, Self::Item>;
    fn draw(&mut self) -> Option<Self::Item> {
        self.remove(self.len() - 1)
    }
    fn peek(&self) -> Option<&Self::Item> {
        self.get(self.len() - 1)
    }
    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let total_iters = self.len();
        for i in 0..total_iters {
            let x = self.remove(rng.gen_range(0..total_iters-i));
            self.add(x);
        }
    }
    fn draw_to<S: Stack<Item=Self::Item>>(&mut self, other: &mut S) {
        if let Some(i) = self.draw() {
            other.add(i);
        }
    }
    fn move_all_to<S: Stack<Item=Self::Item>>(&mut self, other: &mut S) {
        match self.draw() {
            None => (),
            Some(card) => {
                other.add(card);
                self.move_all_to(other)
            }
        }
    }
}

#[derive(Debug)]
pub struct SimpleStack<T> {
    pub elements: Vec<T>,
}

impl<T> SimpleStack<T> {
    pub fn new(elements: Vec<T>) -> SimpleStack<T> {
        SimpleStack {
            elements
        }
    }
    pub fn empty() -> SimpleStack<T> {
        SimpleStack {
            elements: vec![]
        }
    }
}

impl<T> Stack for SimpleStack<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            None
        } else {
            Some(self.elements.remove(index))
        }
    }

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.elements.get(index)
    }

    fn add(&mut self, element: T) {
        self.elements.push(element);
    }
    // fn shuffle(&mut self) {
    //     if self.len() < 2 {
    //         return;
    //     }
    //     let mut new_stack: SimpleStack<T> = SimpleStack::empty();
    //     let mut rng = rand::thread_rng();
    //
    //     // move all the elements into a different stack
    //     let max_len = self.elements.len();
    //     for i in (0..max_len).rev() {
    //         new_stack.add(self.elements.remove(i));
    //     }
    //
    //     // replace them randomly
    //     for i in 0..max_len {
    //         let r = rng.gen_range(0..max_len - i);
    //         self.add(new_stack.elements.remove(r));
    //     }
    // }

    fn iter(&self) -> Iter<'_, Self::Item> {
        self.elements.iter()
    }
}

impl<T: Clone> Clone for SimpleStack<T> {
    fn clone(&self) -> Self {
        SimpleStack::new(self.elements.clone())
    }
}