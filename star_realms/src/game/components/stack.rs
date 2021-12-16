use rand::Rng;
use std::slice::Iter;
use std::collections::HashSet;

pub trait Stack<Item> {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Item>;
    fn iter(&self) -> Iter<'_, Item>;
    fn add(&mut self, item: Item);
    fn remove(&mut self, index: usize) -> Option<Item>;
    fn draw(&mut self) -> Option<Item> {
        self.remove(self.len() - 1)
    }
    fn peek(&self) -> Option<&Item> {
        self.get(self.len() - 1)
    }
    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let total_iters = self.len();
        for i in 0..total_iters {
            let x = self.remove(rng.gen_range(0..total_iters-i)).unwrap();
            self.add(x);
        }
    }
}

pub fn move_all_to<I, S: Stack<I>, T: Stack<I>>(from: &mut T, to: &mut S) {
    match from.draw() {
        None => (),
        Some(card) => {
            to.add(card);
            move_all_to(from, to)
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

impl<T> Stack<T> for SimpleStack<T> {

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

    fn get(&self, index: usize) -> Option<&T> {
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

    fn iter(&self) -> Iter<'_, T> {
        self.elements.iter()
    }
}

impl<T: Clone> Clone for SimpleStack<T> {
    fn clone(&self) -> Self {
        SimpleStack::new(self.elements.clone())
    }
}