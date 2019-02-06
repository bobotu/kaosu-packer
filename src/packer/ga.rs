use rand::prelude::*;

pub type Chromosome = Vec<f32>;

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub population_size: usize,
    pub num_elites: usize,
    pub num_mutants: usize,
    pub inherit_elite_probability: f64,
    pub max_generations: i32,
    pub max_generations_no_improvement: i32,
}

pub trait Decoder {
    type Solution: Clone;

    fn decode_chromosome(&self, individual: &Chromosome) -> Self::Solution;
    fn fitness_of(&self, solution: &Self::Solution) -> f64;
}

pub trait Generator {
    fn generate_individual(&self) -> Chromosome;
}

#[derive(Copy, Clone, Debug)]
pub struct RandGenerator {
    length: usize,
}

impl RandGenerator {
    pub fn new(length: usize) -> RandGenerator {
        RandGenerator { length }
    }
}

impl Generator for RandGenerator {
    fn generate_individual(&self) -> Vec<f32> {
        let mut rng = thread_rng();
        (0..self.length).map(|_| rng.gen()).collect()
    }
}

pub struct Solver<G, D>
where
    G: Generator,
    D: Decoder,
{
    generator: G,
    decoder: D,
    params: Params,
    population: Vec<InnerChromosome<D::Solution>>,
}

impl<G, D> Solver<G, D>
where
    G: Generator,
    D: Decoder,
{
    pub fn new(params: Params, generator: G, decoder: D) -> Solver<G, D> {
        Solver {
            generator,
            decoder,
            params,
            population: Vec::new(),
        }
    }

    pub fn solve(&mut self) -> D::Solution {
        let mut generation = 0;
        let mut generations_no_improvement = 0;

        self.init_population();

        while generation < self.params.max_generations
            && generations_no_improvement < self.params.max_generations_no_improvement
        {
            let new_population = self.evolve_new_generation();

            if new_population[0].fitness < self.population[0].fitness {
                generations_no_improvement = 0;
            } else {
                generations_no_improvement += 1;
            }

            self.population = new_population;
            generation += 1;
        }

        self.population[0].solution.clone()
    }

    fn evolve_new_generation(&mut self) -> Vec<InnerChromosome<D::Solution>> {
        let mut new_population = Vec::with_capacity(self.params.population_size);

        self.fill_elites(&mut new_population);
        self.fill_mutants(&mut new_population);
        self.fill_offsprings(&mut new_population);
        Self::sort_population(&mut new_population);

        new_population
    }

    fn fill_elites(&self, new_population: &mut Vec<InnerChromosome<D::Solution>>) {
        for elite in &self.population[0..self.params.num_elites] {
            new_population.push(elite.clone());
        }
    }

    fn fill_mutants(&self, new_population: &mut Vec<InnerChromosome<D::Solution>>) {
        for _ in 0..self.params.num_mutants {
            new_population.push(self.random_individual());
        }
    }

    fn fill_offsprings(&self, new_population: &mut Vec<InnerChromosome<D::Solution>>) {
        let params = &self.params;
        let num_offsprings = params.population_size - params.num_elites - params.num_mutants;

        for _ in 0..num_offsprings {
            let (elite, non_elite) = self.pickup_parents_for_crossover();
            let offspring = self.crossover(elite, non_elite);
            new_population.push(offspring);
        }
    }

    fn crossover(
        &self,
        elite: &Chromosome,
        non_elite: &Chromosome,
    ) -> InnerChromosome<D::Solution> {
        let mut rng = thread_rng();
        let mut offspring = Vec::with_capacity(elite.len());
        for i in 0..elite.len() {
            let p: f64 = rng.gen();
            let gen = if p <= self.params.inherit_elite_probability {
                elite[i]
            } else {
                non_elite[i]
            };
            offspring.push(gen);
        }
        self.evaluate_chromosome(offspring)
    }

    fn pickup_parents_for_crossover(&self) -> (&Chromosome, &Chromosome) {
        let mut rng = thread_rng();
        let elite_size = self.params.num_elites;
        let non_elite_size = self.params.population_size - elite_size;
        let elite = &self.population[rng.gen_range(0, elite_size)];
        let non_elite = &self.population[elite_size + rng.gen_range(0, non_elite_size)];

        (&elite.chromosome, &non_elite.chromosome)
    }

    fn random_individual(&self) -> InnerChromosome<D::Solution> {
        let chromosome = self.generator.generate_individual();
        self.evaluate_chromosome(chromosome)
    }

    fn evaluate_chromosome(&self, chromosome: Chromosome) -> InnerChromosome<D::Solution> {
        let solution = self.decoder.decode_chromosome(&chromosome);
        let fitness = self.decoder.fitness_of(&solution);
        InnerChromosome {
            chromosome,
            solution,
            fitness,
        }
    }

    fn init_population(&mut self) {
        let mut population = (0..self.params.population_size)
            .into_iter()
            .map(|_| self.random_individual())
            .collect();
        Self::sort_population(&mut population);
        self.population = population;
    }

    fn sort_population(population: &mut Vec<InnerChromosome<D::Solution>>) {
        population.sort_unstable_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
    }
}

#[derive(Clone)]
struct InnerChromosome<S: Clone> {
    chromosome: Chromosome,
    solution: S,
    fitness: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NaiveDecoder;

    impl Decoder for NaiveDecoder {
        type Solution = f64;

        fn decode_chromosome(&self, individual: &Chromosome) -> Self::Solution {
            let mut sum = 0.0;
            for n in individual {
                sum += *n;
            }
            sum as f64
        }

        fn fitness_of(&self, solution: &Self::Solution) -> f64 {
            *solution
        }
    }

    #[test]
    fn naive_test() {
        let params = Params {
            population_size: 10,
            num_elites: 3,
            num_mutants: 2,
            inherit_elite_probability: 0.6,
            max_generations: 10,
            max_generations_no_improvement: 3,
        };

        let mut solver = Solver::new(params, RandGenerator::new(10), NaiveDecoder);
        let _: f64 = solver.solve();
    }
}
