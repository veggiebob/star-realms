use rand::Rng;
use std::slice::Iter;

#[derive(Debug)]
pub struct Stack<T> {
    pub elements: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new(elements: Vec<T>) -> Stack<T> {
        Stack {
            elements
        }
    }

    pub fn empty() -> Stack<T> {
        Stack {
            elements: vec![]
        }
    }

    pub fn peek(&self, index: usize) -> Option<&T> {
        self.elements.get(index)
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn add(&mut self, element: T) {
        self.elements.push(element);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            None
        } else {
            Some(self.elements.remove(index))
        }
    }

    /// we say that the "top" card is the last index card
    pub fn draw(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            Some(self.elements.remove(self.elements.len() - 1))
        }
    }
    pub fn shuffle(&mut self) {
        if self.len() < 2 {
            return;
        }
        let mut new_stack: Stack<T> = Stack::empty();
        let mut rng = rand::thread_rng();

        // move all the elements into a different stack
        let max_len = self.elements.len();
        for i in (0..max_len).rev() {
            new_stack.add(self.elements.remove(i));
        }

        // replace them randomly
        for i in 0..max_len {
            let r = rng.gen_range(0..max_len - i);
            self.add(new_stack.elements.remove(r));
        }
    }

    /// draw a card from self and place it in other
    pub fn draw_to(&mut self, other: &mut Stack<T>) {
        match self.draw() {
            None => (),
            Some(card) => other.add(card),
        }
    }

    pub fn move_all_to(&mut self, other: &mut Stack<T>) {
        match self.draw() {
            None => (),
            Some(card) => {
                other.add(card);
                self.move_all_to(other)
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.elements.iter()
    }
}

impl<T: Clone> Clone for Stack<T> {
    fn clone(&self) -> Self {
        Stack::new(self.elements.clone())
    }
}