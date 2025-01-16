use std::collections::HashMap;
use std::fmt::{self, Formatter, Display};

type HeapNodeId = usize;

// in this code, `floating` is used to represent the state of a node that is not a child of any other node nor set in the rank_id_cache.

pub trait MutableHeap<K: Ord> {
    /// push a new node with key `key` into the heap
    /// and return the `id` of the node
    fn push(&mut self, key: K) -> HeapNodeId;
    /// pop the node with the minimum key and its id from the heap
    fn pop(&mut self) -> Option<(HeapNodeId, K)>;
    /// get the minimum `key` and its `id` from the heap
    fn get_min(&self) -> Option<HeapNodeId>;
    /// modify the key of the node with id `id` to `new_key`
    fn modify(&mut self, id: HeapNodeId, new_key: K);
}

struct Node<K> {
    /// primitive data
    key: K,
    parent: Option<HeapNodeId>,
    children: Vec<HeapNodeId>,
    /// state
    shrinked: bool,
}
impl<K> Node<K> {
    fn rank(&self) -> usize {
        self.children.len()
    }
}

pub struct FibonacciHeap<K: Ord> {
    /// primitive data
    id_node_map: HashMap<HeapNodeId, Node<K>>,
    /// state
    id_provider: usize,
    /// cache
    min_id_cache: Option<HeapNodeId>,
    rank_id_cache: HashMap<usize, HeapNodeId>,
}

/// general methods (methods that are required for multiple MutableHeap methods)
impl<K: Ord> FibonacciHeap<K> {
    pub fn new() -> Self {
        FibonacciHeap {
            id_node_map: HashMap::new(),
            id_provider: 0,
            min_id_cache: None,
            rank_id_cache: HashMap::new(),
        }
    }
    /// take the ids of two heap heads and join them
    /// returns id of head of the merged heap
    fn merge(&mut self, heap1: HeapNodeId, heap2: HeapNodeId) -> HeapNodeId {
        debug_assert!(self.is_valid_as_peek(&heap1));
        debug_assert!(self.is_valid_as_peek(&heap2));
        // heap1 and heap2 are different
        debug_assert!(heap1 != heap2);

        let (smaller, larger): (HeapNodeId, HeapNodeId) = if self.id_node_map.get(&heap1).unwrap().key < self.id_node_map.get(&heap2).unwrap().key {
            (heap1, heap2)
        } else {
            (heap2, heap1)
        };

        let smaller_node = self.id_node_map.get_mut(&smaller).unwrap();
        smaller_node.children.push(larger);
        let larger_node = self.id_node_map.get_mut(&larger).unwrap();
        larger_node.parent = Some(smaller);

        smaller
    }
    /// put the heap keeping fibonacci-heap property. also update rank_id_cache
    fn put(&mut self, heap: HeapNodeId, rank: usize) {
        debug_assert!(self.is_valid_as_peek(&heap));
        // given rank is consistent with the rank of the heap
        // note: although rank can be calculated by heap, it is given as an argument for efficiency
        debug_assert!(self.id_node_map.get(&heap).unwrap().rank() == rank);

        let cached = self.rank_id_cache.remove(&rank);
        if let Some(cached) = cached {
            let merged_heap = self.merge(heap, cached);
            self.put(merged_heap, rank + 1);
        } else {
            self.rank_id_cache.insert(rank, heap);
        }
    }
    fn land_floating_nodes(&mut self, floating: Vec<HeapNodeId>) {
        for node_id in floating.into_iter() {
            let rank = self.id_node_map.get(&node_id).unwrap().rank();
            self.put(node_id, rank);
        }
    }
    /// update min_id_cache with id if necessary
    fn update_min_id_cache(&mut self, id: HeapNodeId) {
        if self.min_id_cache.is_none() {
            self.min_id_cache = Some(id);
            return;
        }
        let min_id = self.min_id_cache.unwrap();
        if self.id_node_map.get(&id).unwrap().key < self.id_node_map.get(&min_id).unwrap().key {
            self.min_id_cache = Some(id);
        }
    }
    /// debug method. returns true if the given id is valid as a peek. peek means the node that is not a child of any other node
    fn is_valid_as_peek(&self, peek: &HeapNodeId) -> bool {
        self.id_node_map.get(peek).is_some_and(|heap| heap.parent.is_none())
    }
}

// following three blocks are separated so that it is easier to understand. there is no more reason to do so.

/// to push
impl<K: Ord> FibonacciHeap<K> {
    fn provide_id(&mut self) -> HeapNodeId {
        self.id_provider += 1;
        self.id_provider
    }
    fn make_and_link_node(&mut self, id:HeapNodeId, key:K) {
        // make brand new node with id
        let node = Node {
            key: key,
            parent: None,
            children: vec![],
            shrinked: false,
        };
        // link id and node
        self.id_node_map.insert(id, node);
    }
}

/// to pop
impl<K: Ord> FibonacciHeap<K> {
    /// returns true if the assertion is satisfied
    /// only used for debugging
    fn pop_assertions(&self) -> bool {
        if let Some(min_id_cache) = self.min_id_cache {
            assert!(self.is_valid_as_peek(&min_id_cache));
            let min_node = self.id_node_map.get(&min_id_cache).unwrap();
            assert!(self.rank_id_cache.get(&min_node.rank()).is_some());
            assert!(self.rank_id_cache.get(&min_node.rank()).is_some_and(|&id| &id == &min_id_cache));
        }
        return true;
    }
    fn pop_min_node_from_cache(&mut self) -> Option<HeapNodeId> {
        match self.min_id_cache.take() {
            Some(min_id) => {
                let min_node = self.id_node_map.get(&min_id).unwrap();
                self.rank_id_cache.remove(&min_node.rank());
                Some(min_id)
            }
            None => None,
        }
    }
    /// cut off children from given node. returns the former children.
    /// children are floating in the air after this method
    fn release_children(&mut self, id: HeapNodeId) -> Vec<HeapNodeId>{
        let node = self.id_node_map.get_mut(&id).unwrap();
        let children = node.children.clone();
        node.children.clear();
        for child_id in children.iter() {
            let child = self.id_node_map.get_mut(child_id).unwrap();
            child.parent = None;
            child.shrinked = false;
        }
        return children;
    }
    fn rebuild_min_id_cache(&mut self) {
        let mut min_id = None;
        for (_, &id) in self.rank_id_cache.iter() {
            if let Some(min_id_unwrapped) = min_id {
                let current_min = self.id_node_map.get(&min_id_unwrapped).unwrap();
                let candidate = self.id_node_map.get(&id).unwrap();
                if current_min.key > candidate.key {
                    min_id = Some(id.clone());
                }
            } else {
                min_id = Some(id);
            }
        }
        self.min_id_cache = min_id;
    }
}

/// to modify
impl<K: Ord> FibonacciHeap<K> {
    /// detach the child from its parent, mark its parent and do cascading cut if necessary.
    /// this method may leave some nodes floating in the air
    /// returns the floating nodes.
    fn cut(&mut self, parent: HeapNodeId, child: HeapNodeId) -> Vec<HeapNodeId> {
        // detach the child from its parent
        let child_node = self.id_node_map.get_mut(&child).unwrap();
        child_node.parent = None;
        child_node.shrinked = false;

        let mut floating = vec![child];

        let parent_node = self.id_node_map.get_mut(&parent).unwrap();
        if parent_node.parent.is_some() {
            // if the parent has a parent, the parent should be marked as shrinked to do cascading cut
            parent_node.shrinked = true;
        }
        else {
            // if the parent has a parent, the parent may be on rank_id_cache.
            // if the parent remains in rank_id_cache, it becomes inconsistent with the actual rank of the node.
            // to avoid this, remove it from rank_id_cache if any.
            let former_rank_of_parent = parent_node.rank();
            let maybe_parent = self.rank_id_cache.get(&former_rank_of_parent);
            if let Some(maybe_parent) = maybe_parent {
                if maybe_parent == &parent {
                    self.rank_id_cache.remove(&former_rank_of_parent);
                    floating.push(parent);
                }
            }
        }
        // remove the child from the children of the parent
        parent_node.children.retain(|&id| id != child);
        let needs_cascading_cut = parent_node.shrinked;
        
        if needs_cascading_cut {
            let parent_of_parent = parent_node.parent.unwrap();
            let newly_floating = self.cut(parent_of_parent, parent);
            floating.extend(newly_floating.into_iter());
        }

        return floating;
    }
    fn heapify_between(&mut self, parent: HeapNodeId, child: HeapNodeId) {
        let parent_node = self.id_node_map.get(&parent).unwrap();
        let child_node = self.id_node_map.get(&child).unwrap();
        if parent_node.key > child_node.key {
            let floating = self.cut(parent, child);
            self.land_floating_nodes(floating);
        }
    }
}

impl<K: Ord> MutableHeap<K> for FibonacciHeap<K> {
    fn push(&mut self, key: K) -> HeapNodeId {
        let id = self.provide_id();
        self.make_and_link_node(id, key);
        self.update_min_id_cache(id);
        self.put(id, 0);
        id
    }
    fn pop(&mut self) -> Option<(HeapNodeId, K)> {
        debug_assert!(self.pop_assertions());

        let min_id = self.pop_min_node_from_cache();
        if min_id.is_none() {
            return None;
        }
        let min_id = min_id.unwrap();

        let floating = self.release_children(min_id);
        self.land_floating_nodes(floating);

        self.rebuild_min_id_cache();

        return match self.id_node_map.remove(&min_id) {
            Some(min_node) => Some((min_id, min_node.key)),
            None => panic!("minimum node is unexpectedly removed in a way"),
        };
    }
    fn get_min(&self) -> Option<HeapNodeId> {
        if self.min_id_cache.is_none() {
            return None;
        }
        let min_id = self.min_id_cache.unwrap();
        return Some(min_id);
    }
    fn modify(&mut self, id: HeapNodeId, new_key: K) {
        // if client not tracks the id properly, they may try to modify a non-existing node
        assert!(self.id_node_map.contains_key(&id));
        
        let node = self.id_node_map.get_mut(&id).unwrap();
        node.key = new_key;
        self.update_min_id_cache(id);

        // make sure the node satisfies the heap property
        // between the node and its parent
        let node = self.id_node_map.get(&id).unwrap();
        let parent_id = node.parent;
        if let Some(parent_id) = parent_id {
            self.heapify_between(parent_id, id);
        }

        // between the node and its children
        let node = self.id_node_map.get(&id).unwrap();
        for child_id in node.children.clone() {
            self.heapify_between(id, child_id);
        }
    }
}

impl<K: Display> Display for Node<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.key)?;
        if self.shrinked {
            write!(f, "*")?;
        }
        write!(f, "\n")
    }
}

impl<K: Display+Ord> FibonacciHeap<K> {
    fn display_tree(&self, id: HeapNodeId, depth: usize, f: &mut Formatter) -> fmt::Result {
        let node = self.id_node_map.get(&id).unwrap();
        for _ in 0..depth {
            write!(f, "| ")?;
        }
        write!(f, "{}:{}", id, node)?;
        for child_id in node.children.iter() {
            self.display_tree(child_id.clone(), depth + 1, f)?;
        }
        Ok(())
    }
}

impl<K: Display+Ord> Display for FibonacciHeap<K> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (_, id) in self.rank_id_cache.iter() {
            self.display_tree(id.clone(), 0, f)?;
        }
        write!(f, "min_id_cache: {:?}", self.min_id_cache)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mutable_heap() {
        let mut heap = FibonacciHeap::<i32>::new();
        println!("{}\n", heap);
        heap.push(1);
        println!("{}\npushed\n", heap);
        heap.push(2);
        println!("{}\npushed\n", heap);
        heap.push(2);
        println!("{}\npushed\n", heap);
        heap.push(3);
        println!("{}\npushed\n", heap);
        let five = heap.push(5);
        println!("{}\npushed\n", heap);
        let eight = heap.push(8);
        println!("{}\npushed\n", heap);
        heap.push(13);
        println!("{}\npushed\n", heap);
        heap.push(21);
        println!("{}\npushed\n", heap);
        let thirty_four = heap.push(34);
        println!("{}\npushed\n", heap);
        heap.push(4);
        println!("{}\npushed\n", heap);
        heap.push(10);
        println!("{}\npushed\n", heap);
        heap.push(11);
        println!("{}\npushed\n", heap);
        heap.push(3);
        println!("{}\npushed\n", heap);

        heap.modify(five, -1);
        println!("{}\nmodified\n", heap);

        let (id, key) = heap.pop().unwrap();
        println!("{}", heap);
        println!("popped (id: {}, key: {})\n",id,key);
        assert_eq!(id, five);
        assert_eq!(key, -1);
        let (id, key) = heap.pop().unwrap();
        println!("{}", heap);
        println!("popped (id: {}, key: {})\n",id,key);
        assert_eq!(key, 1);

        heap.modify(thirty_four, -1);
        println!("{}\nmodified\n", heap);
        heap.modify(eight, 50);
        println!("{}\nmodified\n", heap);

        let mut previous_key = i32::MIN;
        while let Some((id, key)) = heap.pop() {
            println!("{}", heap);
            assert!(previous_key <= key);
            println!("popped (id: {}, key: {})\n",id,key);
            previous_key = key;
        }
    }
}