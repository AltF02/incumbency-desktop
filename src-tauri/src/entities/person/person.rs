use std::{ops::Range, collections::HashMap};
use maplit::hashmap;
use rand::{Rng};
use crate::{common::util::{float_range, percentage_based_output_int, Date}, common::config::Config, game::{generation::{generate_education_level, get_expected_salary_range}}, entities::business::{ProductType, Business}};
use EducationLevel::*;

use super::debt::{Debt};

#[derive(Default, Clone)]
pub struct Birthday {
    day: i32, // 1-30
    month: i32, // 1-12
}

impl Birthday {
    /// Generates a random birthday date
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            day: rng.gen_range(1..=30),
            month: rng.gen_range(1..=12),
        }
    }
}

#[derive(Default, Clone)]
pub struct Person {
    pub id: usize,

    pub education_level: EducationLevel,
    pub years_in_higher_education: i32, // Amount of years the individual spent in college or university (TODO: use this)
    pub job: Job,
    pub debts: Vec<Debt>,

    pub age: i32,
    pub birthday: Birthday,
    pub balance: f32,

    pub expected_salary_range: Range<i32>, // Range of the expected salary for the person based on their education level
    pub salary: i32,

    pub spending_behaviour: SpendingBehaviour,
    pub daily_food_spending: i32,

    pub demand: HashMap<ProductType, f32>,
    pub business_this_month: usize,  // The business the individual will buy from this month, until marketing is re-evaluated
    pub purchase_days: HashMap<i32, i32>, // The days of the month that they will make a purchase - <day, quantity>

    pub health_percentage: i32, // The percentage of their health that they have remaining
    pub hospitalisation_percentage: i32, // The percentage of their health that will require them to be hospitalised
    pub hospitalisation_count: i32, // The amount of times the individual has been hospitalised
    pub days_until_death: Option<i32>, // If the person is predicted to die, use this as a counter
    pub days_left_in_hospital: Option<i32>, // Days left that the person is in hospitalisation
    pub maximum_health: i32,

    pub homeless: bool,
}

impl Person {
    pub fn generate(&mut self, config: &Config, product_demand: &mut HashMap<ProductType, f32>, id: usize) {
        self.id = id;
        self.daily_food_spending = 4; // this is just a default value to prevent bugs
        self.homeless = false;
        self.age = self.generate_age();
        self.generate_health();
        
        self.education_level = generate_education_level(&config);
        self.expected_salary_range = get_expected_salary_range(&config, &self.education_level);

        let expected_salary = ((self.expected_salary_range.start + self.expected_salary_range.end) / 2) as f32;

        self.spending_behaviour = self.generate_spending_behaviour();
        self.balance = self.generate_balance(expected_salary);
        self.birthday = Birthday::generate();
        self.debts = Debt::generate(self, expected_salary);

        self.generate_demand(expected_salary, product_demand);
    }

    fn generate_balance(&self, salary: f32) -> f32 {
        if salary > 0. {
            /*
                We calculate the average % of salary U.S citizens have in their bank account with ((average_salary * us_population) / us_gdp) * 100
                This evaluated to 107%, have added a 50% leeway which gives us a range between 53.5% and 214% of the individuals salary
            */
            return salary * float_range(0.535, 2.14, 3);
        }


        // TODO: make this depend on spending behaviour
        float_range(50., 1200., 3)
    }

    fn generate_spending_behaviour(&self) -> SpendingBehaviour {
        if matches!(self.job, Job::BusinessOwner(_)) {
            return percentage_based_output_int::<SpendingBehaviour>(hashmap! {
                SpendingBehaviour::One => 1,
                SpendingBehaviour::Two => 4,
                SpendingBehaviour::Three => 25,
                SpendingBehaviour::Four => 70,
            });
        }

        match self.education_level {
            NoFormalEducation => percentage_based_output_int::<SpendingBehaviour>(hashmap! {
                SpendingBehaviour::One => 75,
                SpendingBehaviour::Two => 20,
                SpendingBehaviour::Three => 4,
                SpendingBehaviour::Four => 1,
            }),

            HighSchoolDiploma => percentage_based_output_int::<SpendingBehaviour>(hashmap! {
                SpendingBehaviour::One => 20,
                SpendingBehaviour::Two => 70,
                SpendingBehaviour::Three => 9,
                SpendingBehaviour::Four => 1,
            }),

            College | AssociateDegree => percentage_based_output_int::<SpendingBehaviour>(hashmap! {
                SpendingBehaviour::One => 3,
                SpendingBehaviour::Two => 10,
                SpendingBehaviour::Three => 82,
                SpendingBehaviour::Four => 5,
            }),

            Bachelors | AdvancedDegree => percentage_based_output_int::<SpendingBehaviour>(hashmap! {
                SpendingBehaviour::One => 1,
                SpendingBehaviour::Two => 4,
                SpendingBehaviour::Three => 77,
                SpendingBehaviour::Four => 18,
            })
        }
    }

    fn generate_age(&self) -> i32 {
        let mut rng = rand::thread_rng();

        percentage_based_output_int::<i32>(hashmap! {
            (rng.gen_range(0..=18)) => 24,
            (rng.gen_range(19..=25)) => 9,
            (rng.gen_range(26..=34)) => 12,
            (rng.gen_range(35..=54)) => 25,
            (rng.gen_range(55..=64)) => 13,
            (rng.gen_range(65..=90)) => 17,
        })
    }

    fn generate_demand(&mut self, salary: f32, product_demand: &mut HashMap<ProductType, f32>) {
        // TODO: generate based on more factors rather than just salary
        let mut rng = rand::thread_rng();

        // The percentage of balance that will be added to demand
        let (balance_percentage, salary_percentage) = match self.spending_behaviour {
            SpendingBehaviour::One => (float_range(0.4, 1., 2), rng.gen_range(1..5) as f32),
            SpendingBehaviour::Two => (float_range(0.08, 0.3, 3), rng.gen_range(1..=3) as f32),
            SpendingBehaviour::Three =>  (float_range(0.02, 0.25, 3), float_range(0.01, 0.05, 2)),
            SpendingBehaviour::Four => (float_range(0.005, 0.058, 4), 0.),
        };

        let mut total_demand = self.balance * (balance_percentage as f32 / 100.);
        
        total_demand += (salary / 12.) * (salary_percentage as f32 / 100.);

        self.demand.insert(ProductType::LEISURE, total_demand);
        *product_demand.get_mut(&ProductType::LEISURE).unwrap() += total_demand;
    }

    pub fn calculate_daily_food_spending(&self) -> i32 {
        let debt_cost = self.get_monthly_debt_cost();

        let healthy_cost = debt_cost + (4 * 30) as f32;
        let survivable_cost = debt_cost + (3 * 30) as f32;
        let unhealthy_cost = debt_cost + (2 * 30) as f32;

        if self.can_afford_bare(healthy_cost) {
            return 4;
        } else if self.can_afford_bare(survivable_cost) {
            return 3;
        } else if self.can_afford_bare(unhealthy_cost) {
            let (action_one_chance, action_two_chance) = match self.spending_behaviour {
                SpendingBehaviour::One => (90, 10),
                SpendingBehaviour::Two => (55, 45),
                SpendingBehaviour::Three => (35, 65),
                SpendingBehaviour::Four => (10, 90),
            };

            let action = percentage_based_output_int(hashmap! {
                1 => action_one_chance,
                2 => action_two_chance,
            });

            if action == 1 {
                return 3;
            } else {
                return 2;
            }
        }

        1
    }

    /// This should be done every time the individual's salary changes, and every month.
    pub fn generate_daily_food_spending(&mut self) {
        match self.job {
            Job::BusinessOwner(_) => return, // TODO: implement me
            _ => (), 
        }
        
        self.daily_food_spending = self.calculate_daily_food_spending();

        // 0.17% chance of being homeless
        let mut rng = rand::thread_rng();
        let maximum = (100. / 0.17) as i32; 
        let is_homeless = rng.gen_range(0..=maximum) == maximum;
        self.homeless = is_homeless;

        // TODO: handle unemployed
    }

    pub fn can_afford(&self, price: f32) -> bool {
        let mut cut_balance: f32 = self.balance - (self.balance * 0.1) - ((self.salary as f32 / 12.) * 0.1);
        cut_balance -= self.get_monthly_debt_cost() as f32;

        cut_balance -= self.daily_food_spending as f32 * 30.;
        cut_balance - price > 0.
    }

    pub fn can_afford_bare(&self, price: f32) -> bool {
        self.balance - price > 0.
    }

    pub fn business_pay(&mut self, payee: &mut Business, amount: f32) {
        self.balance += amount;
        payee.balance -= amount;
    }
    
    pub fn pay_tax(&mut self, government_balance: &mut u128, amount: f32) {
        if amount < 0. { return }
        self.balance -= amount;
        *government_balance += amount as u128;
    }

    pub fn check_birthday(&mut self, date: &Date) {
        if date.day == self.birthday.day && date.month == self.birthday.month {
            self.grow_up();
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub enum EducationLevel {
    #[default]
    NoFormalEducation,
    HighSchoolDiploma,
    College,
    AssociateDegree,
    Bachelors,
    AdvancedDegree
}

#[derive(Default, Clone, PartialEq)]
pub enum Job {
    BusinessOwner(usize), // usize refers to index of the business in the game state
    Employee(usize),
    #[default]
    Unemployed,
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub enum SpendingBehaviour {
    #[default]
    One,
    Two,
    Three,
    Four,
}