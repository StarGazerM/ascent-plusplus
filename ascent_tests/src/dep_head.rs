

use ascent::*;
use rayon::{iter::Empty, vec};
use std::{borrow::Borrow, collections::BTreeSet, rc::Rc};

type item_type = i32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Bag {
    Empty,
    // we can put an item in the bag, with its value and weight
    Cons(usize, Rc<Bag>),
}

type BagSet = Vec<usize>;


// fn total_bag(b: &Rc<Bag>) -> i32 {
//     match &**b {
//         Bag::Empty => 0,
//         Bag::Cons(_, x, xs) => x + total_bag(xs),
//     }
// }

// fn in_bag(b: &Rc<Bag>, x: i32) -> bool {
//     match &**b {
//         Bag::Empty => false,
//         Bag::Cons(y, _, xs) => x == *y || in_bag(xs, x),
//     }
// }

fn vec_insert(x: usize, xs: &Rc<BagSet>) -> Rc<BagSet> {
    let mut new_bag = (**xs).clone();
    // binary find the position to insert x
    let pos = new_bag.binary_search(&x).unwrap_or_else(|x| x);
    new_bag.insert(pos, x);
    Rc::new(new_bag)
}

// ; Given a capacity and a list of objects, finds the maximum value of a
// ; collection of objects whose total weight does not exceed the capacity.
#[test]
fn knapsack_ascent() {
    let max_size = 10;
    let max_weight = 100;
    let max_select = 5;
    let test_input = vec![
        (5, 10),
        (4, 40),
        (6, 30),
        (3, 50),
        (2, 10),
        (2, 20),
    ];

    let res = ascent_run! {
        // relation max_capacity(i32);

        // the item you can put in the knapsack, with its id, weight and value
        relation input_items(usize, i32, i32);

        input_items(i, w, v) <--
            for i in 0..test_input.len(),
            let (w, v) = test_input[i];
        // relation input_size(usize);

        // select item k at step n
        relation select_item_weight(usize, Rc<BagSet>, i32, i32);
        lattice heaviest_bag(i32);
        heaviest_bag(0);

        select_item_weight(i, init_bag.clone(), 0, 0) <--
            for i in 0..max_size,
            let init_bag = Rc::new(vec![]);

        select_item_weight(j, new_bag, new_weight, new_value) <--
            select_item_weight(i, bag, weight, old_value),
            for j in 0..max_size,
            input_items(j, w, value),
            if bag.binary_search(&j).is_err(),
            let new_weight = weight + w,
            if new_weight <= max_weight && bag.len() < max_select,
            let new_bag = vec_insert(j, &bag),
            let new_value = old_value + value;

        heaviest_bag(value) <--
            select_item_weight(i, bag, weight, value),
            for j in 0..max_size,
            input_items(j, w, value),
            if bag.binary_search(&j).is_err(),
            let new_weight = weight + w,
            if new_weight > max_weight || bag.len() >= max_select;

        // heaviest_bag(bag, weight) 
        
    };

    println!("{:?}", res.heaviest_bag);
    println!("{:?}", res.select_item_weight);
}


