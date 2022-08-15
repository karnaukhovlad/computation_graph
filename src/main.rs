use std::cell::{Ref, RefCell};
use std::convert::AsRef;
use std::mem;
use std::rc::Rc;

struct Node(Rc<RefCell<NodeInner>>);

impl Node {
    pub fn new(func: fn(Vec<f32>) -> Vec<f32>) -> Self {
        Self(Rc::new(RefCell::new(NodeInner::new(func))))
    }

    pub fn input(&self) -> Input {
        Input {
            reference: self.0.clone(),
        }
    }

    fn add_children(&mut self, children: &mut Node) {
        let mut self_br_mut = self.as_ref().borrow_mut();
        self_br_mut.down.push(Node(children.0.clone()));
        children.as_ref().borrow_mut().up.push(Node(self.0.clone()));

        self_br_mut.clear_cache();
    }

    pub fn compute(&mut self) -> Ref<'_, [f32]> {
        {
            let mut guard = self.as_ref().borrow_mut();
            guard.compute();
        }
        Ref::map(self.0.as_ref().borrow(), |inner| inner.output())
    }
}

impl AsRef<RefCell<NodeInner>> for Node {
    fn as_ref(&self) -> &RefCell<NodeInner> {
        self.0.as_ref()
    }
}

struct NodeInner {
    // Instead Vec we can use HashMap to exclude duplication and better handle relationship.
    up: Vec<Node>,
    down: Vec<Node>,
    // Instead this function signature we can use fn(f32, f32) -> f32 that exclude handling existence of the element,
    // but then we need more nodes for cases with multiply inputs,outputs.
    func: fn(Vec<f32>) -> Vec<f32>,
    cache: Option<Vec<f32>>,
    input: Option<Vec<f32>>,
}

impl NodeInner {
    fn new(func: fn(Vec<f32>) -> Vec<f32>) -> Self {
        Self {
            up: vec![],
            down: vec![],
            func,
            cache: None,
            input: None,
        }
    }

    fn compute(&mut self) {
        if self.cache.is_none() {
            let input = self
                .down
                .iter()
                .map(|node| {
                    let mut refer = node.as_ref().borrow_mut();
                    refer.compute();
                    refer.output().to_owned()
                })
                .flatten()
                .chain(self.input.as_ref().unwrap_or(&vec![]).iter().cloned())
                .collect();
            let result = (self.func)(input);
            self.cache = Some(result);
        };
    }

    fn output(&self) -> &[f32] {
        match self.cache {
            None => {
                unreachable!()
            }
            Some(ref res) => return res.as_slice(),
        };
    }

    #[allow(dead_code)]
    fn clear_cache(&mut self) {
        if let Some(cleared) = mem::take(&mut self.cache) {
            println!("Cache cleared: {:?}", cleared);
        };

        self.up
            .iter_mut()
            .for_each(|node| node.as_ref().borrow_mut().clear_cache());
    }
}

struct Input {
    reference: Rc<RefCell<NodeInner>>,
}

impl Input {
    #[allow(dead_code)]
    pub fn get(&self) -> Ref<'_, Option<Vec<f32>>> {
        Ref::map(self.reference.as_ref().borrow(), |node_inner| {
            &node_inner.input
        })
    }
    pub fn set(&self, input: Vec<f32>) {
        let mut br_mut = self.reference.as_ref().borrow_mut();
        br_mut.input = Some(input);
        br_mut.clear_cache();
    }

    #[allow(dead_code)]
    pub fn insert(&self, index: usize, value: f32) -> Option<()> {
        let mut br_mut = self.reference.as_ref().borrow_mut();
        match br_mut.input {
            None => None,
            Some(ref mut input) => {
                input.insert(index, value);
                br_mut.clear_cache();
                Some(())
            }
        }
    }
}

#[allow(dead_code)]
fn round(x: f32, precision: u32) -> f32 {
    let m = 10i32.pow(precision) as f32;
    (x * m).round() / m
}

fn main() {
    let mut node_1 = Node::new(|input| vec![input.get(0).unwrap().powf(3.0)]);
    let mut node_2 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);
    let mut node_3 = Node::new(|input| vec![input.get(0).unwrap().sin()]);
    let mut node_4 = Node::new(|input| vec![input.get(0).unwrap() * input.get(1).unwrap()]);
    let mut node_5 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);

    let node_1_input = node_1.input();
    let node_2_input = node_2.input();
    let node_4_input = node_4.input();
    let node_5_input = node_5.input();

    node_1_input.set(vec![3.0]);
    node_2_input.set(vec![2.0]);
    node_4_input.set(vec![2.0]);
    node_5_input.set(vec![1.0]);

    node_2.add_children(&mut node_1);
    node_3.add_children(&mut node_2);
    node_4.add_children(&mut node_3);
    node_5.add_children(&mut node_4);

    let output = node_5.compute();

    println!("Output: {:?}", &output);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1() {
        let mut node_1 = Node::new(|input| vec![input.get(0).unwrap().powf(3.0)]);
        let mut node_2 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);
        let mut node_3 = Node::new(|input| vec![input.get(0).unwrap().sin()]);
        let mut node_4 = Node::new(|input| vec![input.get(0).unwrap() * input.get(1).unwrap()]);
        let mut node_5 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);

        let node_1_input = node_1.input();
        let node_2_input = node_2.input();
        let node_4_input = node_4.input();
        let node_5_input = node_5.input();

        node_1_input.set(vec![3.0]);
        node_2_input.set(vec![2.0]);
        node_4_input.set(vec![2.0]);
        node_5_input.set(vec![1.0]);

        node_2.add_children(&mut node_1);
        node_3.add_children(&mut node_2);
        node_4.add_children(&mut node_3);
        node_5.add_children(&mut node_4);

        let output = node_5.compute();

        assert_eq!(round(output[0], 5), -0.32727);
    }

    #[test]
    fn test_2() {
        let mut node_1 = Node::new(|input| vec![input.get(0).unwrap().powf(3.0)]);
        let mut node_2 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);
        let mut node_3 = Node::new(|input| vec![input.get(0).unwrap().sin()]);
        let mut node_4 = Node::new(|input| vec![input.get(0).unwrap() * input.get(1).unwrap()]);
        let mut node_5 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);

        let mut node_input_1 = Node::new(|input| input);
        let mut node_input_2 = Node::new(|input| input);
        let mut node_input_3 = Node::new(|input| input);

        let input_1 = node_input_1.input();
        let input_2 = node_input_2.input();
        let input_3 = node_input_3.input();

        input_1.set(vec![1.0]);
        input_2.set(vec![2.0]);
        input_3.set(vec![3.0]);

        node_1.add_children(&mut node_input_3);

        node_2.add_children(&mut node_1);
        node_2.add_children(&mut node_input_2);

        node_3.add_children(&mut node_2);

        node_4.add_children(&mut node_3);
        node_4.add_children(&mut node_input_2);

        node_5.add_children(&mut node_4);
        node_5.add_children(&mut node_input_1);

        let output = node_5.compute();

        assert_eq!(round(output[0], 5), -0.32727);
    }

    #[test]
    fn test_3() {
        let mut node_1 = Node::new(|input| vec![input.get(0).unwrap().powf(3.0)]);
        let mut node_2 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);
        let mut node_3 = Node::new(|input| vec![input.get(0).unwrap().sin()]);
        let mut node_4 = Node::new(|input| vec![input.get(0).unwrap() * input.get(1).unwrap()]);
        let mut node_5 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);

        let mut node_input_1 = Node::new(|input| input);
        let mut node_input_2 = Node::new(|input| input);
        let mut node_input_3 = Node::new(|input| input);

        let input_1 = node_input_1.input();
        let input_2 = node_input_2.input();
        let input_3 = node_input_3.input();

        input_1.set(vec![2.0]);
        input_2.set(vec![3.0]);
        input_3.set(vec![4.0]);

        node_1.add_children(&mut node_input_3);

        node_2.add_children(&mut node_1);
        node_2.add_children(&mut node_input_2);

        node_3.add_children(&mut node_2);

        node_4.add_children(&mut node_3);
        node_4.add_children(&mut node_input_2);

        node_5.add_children(&mut node_4);
        node_5.add_children(&mut node_input_1);

        let output = node_5.compute();

        assert_eq!(round(output[0], 5), -0.56656);
    }

    #[test]
    #[should_panic(expected = "already borrowed: BorrowMutError")]
    fn test_4() {
        let mut node_1 = Node::new(|input| vec![input.get(0).unwrap().powf(3.0)]);
        let mut node_2 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);

        node_1.add_children(&mut node_2);
        node_2.add_children(&mut node_1);

        node_2.compute();
    }

    #[test]
    fn test_cache_invalidation() {
        let mut node_1 = Node::new(|input| vec![input.get(0).unwrap().powf(3.0)]);
        let mut node_2 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);
        let mut node_3 = Node::new(|input| vec![input.get(0).unwrap().sin()]);
        let mut node_4 = Node::new(|input| vec![input.get(0).unwrap() * input.get(1).unwrap()]);
        let mut node_5 = Node::new(|input| vec![input.get(0).unwrap() + input.get(1).unwrap()]);

        let mut node_input_1 = Node::new(|input| input);
        let mut node_input_2 = Node::new(|input| input);
        let mut node_input_3 = Node::new(|input| input);

        let input_1 = node_input_1.input();
        let input_2 = node_input_2.input();
        let input_3 = node_input_3.input();

        input_1.set(vec![2.0]);
        input_2.set(vec![3.0]);
        input_3.set(vec![4.0]);

        node_1.add_children(&mut node_input_3);

        node_2.add_children(&mut node_1);
        node_2.add_children(&mut node_input_2);

        node_3.add_children(&mut node_2);

        node_4.add_children(&mut node_3);
        node_4.add_children(&mut node_input_2);

        node_5.add_children(&mut node_4);
        node_5.add_children(&mut node_input_1);

        {
            let output = node_5.compute();
            assert_eq!(round(output[0], 5), -0.56656);
        }

        input_1.set(vec![3.0]);

        let output = node_5.compute();
        assert_eq!(round(output[0], 5), 0.43344);
    }
}
