/*
 * Copyright 2019 Zejun Li
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::mem;

use rand::prelude::*;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

pub type Chromosome = Vec<f32>;

#[derive(Clone)]
struct InnerChromosome<S: Clone> {
    chromosome: Chromosome,
    solution: S,
    fitness: f64,
}

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
    #[cfg(not(feature = "rayon"))]
    type Solution: Clone;

    #[cfg(feature = "rayon")]
    type Solution: Clone + Sync + Send;

    fn decode_chromosome(&mut self, individual: &Chromosome) -> Self::Solution;
    fn fitness_of(&self, solution: &Self::Solution) -> f64;
    fn reset(&mut self);
}

#[cfg(feature = "rayon")]
pub trait Generator: Sync + Send {
    fn generate_individual(&self) -> Chromosome;
}

#[cfg(not(feature = "rayon"))]
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

pub struct Solver<G, D, F>
where
    G: Generator,
    D: Decoder,
    F: Fn() -> D,
{
    generator: G,
    decoder_factory: F,
    params: Params,

    // reuse population vec between generations.
    population: Vec<InnerChromosome<D::Solution>>,
    population1: Vec<InnerChromosome<D::Solution>>,
}

macro_rules! define_solve_and_new {
    () => {
        pub fn new(params: Params, generator: G, decoder_factory: F) -> Solver<G, D, F> {
            Solver {
                generator,
                decoder_factory,
                params,
                population: Vec::with_capacity(params.population_size),
                population1: Vec::with_capacity(params.population_size),
            }
        }

        pub fn solve(&mut self) -> D::Solution {
            let mut generation = 0;
            let mut generations_no_improvement = 0;

            self.init_first_generation();

            while generation < self.params.max_generations
                && generations_no_improvement < self.params.max_generations_no_improvement
            {
                let prev_fitness = self.population[0].fitness;
                self.evolve_new_generation();
                let curr_fitness = self.population[0].fitness;

                if curr_fitness < prev_fitness {
                    generations_no_improvement = 0;
                } else {
                    generations_no_improvement += 1;
                }

                generation += 1;
            }

            self.population[0].solution.clone()
        }
    };
}

impl<G, D, F> Solver<G, D, F>
where
    G: Generator,
    D: Decoder,
    F: Fn() -> D,
{
    #[inline]
    fn crossover(
        &self,
        elite: &Chromosome,
        non_elite: &Chromosome,
        rng: &mut ThreadRng,
    ) -> Chromosome {
        let mut offspring = Vec::with_capacity(elite.len());
        offspring.extend((0..elite.len()).map(|i| {
            let p: f64 = rng.gen();
            if p <= self.params.inherit_elite_probability {
                elite[i]
            } else {
                non_elite[i]
            }
        }));
        offspring
    }

    #[inline]
    fn pickup_parents_for_crossover(&self, rng: &mut ThreadRng) -> (&Chromosome, &Chromosome) {
        let elite_size = self.params.num_elites;
        let non_elite_size = self.params.population_size - elite_size;
        let elite = &self.population[rng.gen_range(0, elite_size)];
        let non_elite = &self.population[elite_size + rng.gen_range(0, non_elite_size)];

        (&elite.chromosome, &non_elite.chromosome)
    }

    #[inline]
    fn sort_population(population: &mut Vec<InnerChromosome<D::Solution>>) {
        population.sort_unstable_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
    }

    #[inline]
    fn decode_chromosome(decoder: &mut D, chromosome: Chromosome) -> InnerChromosome<D::Solution> {
        let solution = decoder.decode_chromosome(&chromosome);
        let fitness = decoder.fitness_of(&solution);
        decoder.reset();

        InnerChromosome {
            chromosome,
            solution,
            fitness,
        }
    }
}

#[cfg(feature = "rayon")]
impl<G, D, F> Solver<G, D, F>
where
    G: Generator,
    D: Decoder,
    F: Fn() -> D + Sync + Send,
{
    define_solve_and_new!();

    fn evolve_new_generation(&mut self) {
        let num_elites = self.params.num_elites;
        let num_mutants = self.params.num_mutants;
        let num_offsprings = self.params.population_size - num_elites - num_mutants;

        let decoder_factory = &self.decoder_factory;
        let generator = &self.generator;
        let mut dummy = Vec::new();

        // reuse decoder in mutant and crossover.
        mem::swap(&mut dummy, &mut self.population1);
        (0..(num_mutants + num_offsprings))
            .into_par_iter()
            .map_init(
                || (decoder_factory(), thread_rng()),
                |&mut (ref mut decoder, ref mut rng), i| {
                    if i < num_mutants {
                        Self::decode_chromosome(decoder, generator.generate_individual())
                    } else {
                        let (elite, non_elite) = self.pickup_parents_for_crossover(rng);
                        let offspring = self.crossover(elite, non_elite, rng);
                        Self::decode_chromosome(decoder, offspring)
                    }
                },
            )
            .collect_into_vec(&mut dummy);
        mem::swap(&mut dummy, &mut self.population1);

        // copy elites (must after collect_into_vec)
        for elite in &self.population[0..num_elites] {
            self.population1.push(elite.clone());
        }

        // sort the new generation and swap backend vec.
        Self::sort_population(&mut self.population1);
        // TODO: we can reuse the memory of individual's vector inside population vector.
        self.population.clear();
        mem::swap(&mut self.population, &mut self.population1);
    }

    fn init_first_generation(&mut self) {
        let decoder_factory = &self.decoder_factory;
        let generator = &self.generator;
        (0..self.params.population_size)
            .into_par_iter()
            .map_init(decoder_factory, |decoder, _| {
                Self::decode_chromosome(decoder, generator.generate_individual())
            })
            .collect_into_vec(&mut self.population);
        Self::sort_population(&mut self.population);
    }
}

#[cfg(not(feature = "rayon"))]
impl<G, D, F> Solver<G, D, F>
where
    G: Generator,
    D: Decoder,
    F: Fn() -> D,
{
    define_solve_and_new!();

    fn init_first_generation(&mut self) {
        let mut decoder = (self.decoder_factory)();
        let generator = &self.generator;
        self.population.extend(
            (0..self.params.population_size)
                .map(|_| Self::decode_chromosome(&mut decoder, generator.generate_individual())),
        );
        Self::sort_population(&mut self.population);
    }

    fn evolve_new_generation(&mut self) {
        let mut decoder = (self.decoder_factory)();
        let mut rng = thread_rng();
        let num_elites = self.params.num_elites;
        let num_mutants = self.params.num_mutants;
        let num_offsprings = self.params.population_size - num_elites - num_mutants;

        // copy elites to next generation.
        for elite in &self.population[0..num_elites] {
            self.population1.push(elite.clone());
        }

        // generate mutants from generator.
        for _ in 0..num_mutants {
            let mutant = self.generator.generate_individual();
            let mutant = Self::decode_chromosome(&mut decoder, mutant);
            self.population1.push(mutant);
        }

        // crossover offsprings.
        for _ in 0..num_offsprings {
            let (elite, non_elite) = self.pickup_parents_for_crossover(&mut rng);
            let offspring = self.crossover(elite, non_elite, &mut rng);
            self.population1
                .push(Self::decode_chromosome(&mut decoder, offspring));
        }

        // sort the new generation and swap backend vec.
        Self::sort_population(&mut self.population1);
        // TODO: we can reuse the memory of individual's vector inside population vector.
        self.population.clear();
        mem::swap(&mut self.population, &mut self.population1);
    }
}
