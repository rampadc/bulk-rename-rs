pub trait Mutation {
    fn mutate(&self, input: &str) -> String;
}

pub struct FunctionMutation<F> {
    function: F,
}

impl<F: Fn(&str) -> String> Mutation for FunctionMutation<F> {
    fn mutate(&self, input: &str) -> String {
        (self.function)(input)
    }
}

pub struct MutationPipeline {
    mutations: Vec<Box<dyn Mutation>>,
}

impl MutationPipeline {
    pub fn new() -> Self {
        Self { mutations: vec![] }
    }

    pub fn add_mutation(&mut self, mutation: Box<dyn Mutation>) {
        self.mutations.push(mutation);
    }

    pub fn apply_mutation(&self, input: &str) -> String {
        // fold: useful when you have a collection of something and want to produce a single value from it
        // fold() takes two arguments: an initial value, and a closure with two arguments:
        //      an ‘accumulator’, and an element.
        // The closure returns the value that the accumulator should have for the next iteration.
        // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold
        // takes in the initial value
        self.mutations.iter().fold(
            input.to_string(), |acc, m| m.mutate(&acc))
    }
}