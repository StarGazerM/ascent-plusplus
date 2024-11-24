

use ascent::*;
use rayon::{iter::Empty, vec};
use std::{borrow::Borrow, collections::{BTreeMap, BTreeSet}, rc::Rc};

type item_type = i32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Bag {
    Empty,
    // we can put an item in the bag, with its value and weight
    Cons(usize, Rc<Bag>),
}

type BagSet = BTreeSet<usize>;

fn vec_insert(x: usize, xs: &Rc<BagSet>) -> Rc<BagSet> {
    let mut new_bag = (**xs).clone();
    // binary find the position to insert x
    new_bag.insert(x);
    Rc::new(new_bag)
}

fn knapsack_normal(max_weight: i32, max_select: usize, test_input: &Vec<(i32, i32)>) -> i32 {
    let mut most_value_bag_size = 0;

    // a map from each BagSet to its value as working set
    let mut bag_weight_value: BTreeMap<Rc<BagSet>, (i32, i32)> = BTreeMap::new();
    bag_weight_value.insert(Rc::new(BTreeSet::default()), (0, 0));
    let mut most_value_bag = Default::default();

    // solve knapsack problem with dynamic programming in rust
    for _ in 0..max_select+1 {
        // start selecting items from any random start
        let mut new_bag_values: Vec<(Rc<BagSet>, i32, i32)> = vec![];
        let mut full_bag : Vec<Rc<BagSet>> = vec![];
        for (bag, (weight, value)) in bag_weight_value.iter() {
            for j in 0..test_input.len() {
                let (w, v) = test_input[j];
                if !bag.contains(&j) {
                    let new_weight = weight + w;
                    if new_weight <= max_weight && bag.len() <= max_select {
                        let new_bag = vec_insert(j, bag);
                        let new_value = value + v;
                        new_bag_values.push((new_bag, new_weight, new_value));
                    } else {
                        full_bag.push(bag.clone());
                        // println!("full bag: {:?}", bag);
                        if value > &most_value_bag_size {
                            most_value_bag_size = *value;
                            most_value_bag = bag.clone();
                        }
                    }
                }
            }
        }
        // purge the full bags from working set
        for bag in full_bag {
            bag_weight_value.remove(&bag);
        }
        // insert the new bags into working set
        for (bag, weight, value) in new_bag_values {
            bag_weight_value.insert(bag, (weight, value));
        }
    }
    // print most value bag
    let mut weight = 0;
    for i in most_value_bag.iter() {
        print!("{:?} ", test_input[*i]);
        print!(" \n");
        weight += test_input[*i].0;
    }
    println!("Total weight: {:?}", weight);
    most_value_bag_size
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BagValue {
    weight: i32,
    value: i32,
    bag: Rc<BagSet>,
}

impl PartialOrd for BagValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl Ord for BagValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Lattice for BagValue {
    fn join_mut(&mut self, other: Self) -> bool {
        if self.value < other.value {
            *self = other;
            true
        } else {
            false
        }
    }
    
    fn meet_mut(&mut self, other: Self) -> bool {
        if self.value > other.value {
            *self = other;
            true
        } else {
            false
        }
    }
}

// ; Given a capacity and a list of objects, finds the maximum value of a
// ; collection of objects whose total weight does not exceed the capacity.
#[test]
fn knapsack_ascent() {
    let max_weight = 100;
    let max_select = 5;
    // random generate 20 input items weight in range (0, 100),
    // value in range (0, 10)
    let test_size = 20;
    let test_input = (0..test_size)
        .map(|_| (rand::random::<i32>() % 100, rand::random::<i32>() % 10))
        .map(|(w, v)| (w.abs(), v.abs()))
        .collect::<Vec<(i32, i32)>>();

    println!("Test Input {} : {:?}", test_size, &test_input);

    let res = knapsack_normal(max_weight, max_select, &test_input);
    println!("{:?}", res);

    let res = ascent_run! {
        // relation max_capacity(i32);

        // the item you can put in the knapsack, with its id, weight and value
        relation input_items(usize, i32, i32);

        input_items(i, w, v) <--
            for i in 0..test_input.len(),
            let (w, v) = test_input[i];
        // relation input_size(usize);

        // select item k at step n
        relation select_item_weight(Rc<BagSet>, i32, i32);
        lattice most_value_bag(BagValue);
        most_value_bag(init_v) <-- let init_v = BagValue { weight: 0, value: 0, bag: Default::default() };

        select_item_weight(Default::default(), 0, 0);

        select_item_weight(new_bag.clone(), new_weight, new_value) <--
            select_item_weight(bag, weight, value),
            for i in 0..test_input.len(),
            if !bag.contains(&i),
            input_items(i, w, v),
            let new_weight = weight + w,
            if new_weight <= max_weight && bag.len() < max_select,
            let new_bag = vec_insert(i, &bag),
            let new_value = value + v;

        most_value_bag(new_value_bag) <--
            select_item_weight(bag, weight, value),
            for i in 0..test_input.len(),
            if !bag.contains(&i),
            input_items(i, w, _),
            let new_weight = weight + w,
            if new_weight > max_weight || bag.len() >= max_select,
            let new_value_bag = BagValue { weight: *weight, value: *value, bag: bag.clone() };
    };

    println!("{:?}", &res.most_value_bag);
    for i in (*res.most_value_bag[0].0.bag).clone() {
        print!("{:?} ", test_input[i]);
        print!(" \n");
    }
    // println!("{:?}", res.select_item_weight);
}


