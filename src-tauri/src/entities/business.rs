use maplit::hashmap;
use rand::Rng;

use crate::{generation::generate_education_level, config::Config, util::{percentage_based_output_int, float_range}};
use super::person::EducationLevel::{*, self};

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub enum ProductType {
    #[default]
    LEISURE,

    // These will be implemented later:
    // FURNITURE
    // HOUSES
}

#[derive(Default)]
pub struct Business {
    pub minimum_education_level: EducationLevel,
    pub expected_marketing_reach: i32, // Amount of population that the marketing will reach (roughly)
}

impl Business {
    /// Generates a business based on demand
    pub fn generate(&mut self, config: &Config, product_type: ProductType, product_demand: f32, remaining_market_percentage: &mut f32) -> bool {
        self.minimum_education_level = generate_education_level(&config);

        let marketing_reach_percentage = match self.minimum_education_level {
            NoFormalEducation => self.random_marketing_percentage_multiplyer(0.1, 0.3),
            HighSchoolDiploma => self.random_marketing_percentage_multiplyer(0.1, 0.4),
            College => self.random_marketing_percentage_multiplyer(0.4, 0.7),
            AssociateDegree => self.random_marketing_percentage_multiplyer(0.5, 1.),
            Bachelors => self.random_marketing_percentage_multiplyer(0.7, 1.1),
            AdvancedDegree => self.random_marketing_percentage_multiplyer(0.8, 1.5),
        } as f32;

        if (*remaining_market_percentage - marketing_reach_percentage) < 0. {
            return false;
        }

        *remaining_market_percentage -= marketing_reach_percentage;

        // TODO: make this better
        let product_price = rand::thread_rng().gen_range(2..100) as f32;

        // self.expected_marketing_reach = ((marketing_reach_percentage / 100.) as f32 * (product_demand / product_price)) as i32; - this is actually the required stock
        return true;
    }


    /// Multiplies the percentage target audience for the market based on educated odds 
    pub fn random_marketing_percentage_multiplyer(&self, min: f32, max: f32) -> f32 {
        // 1 - smallest, 3 - largest
        let tier = percentage_based_output_int(hashmap! {
            1 => 75,
            2 => 20,
            3 => 5,
        });

        let mut rng = rand::thread_rng();
        let increase_multiplyer = match tier {
            2 => rng.gen_range(150..320) as f32, // Increase start and end by a random range of 150%-320%
            3 => rng.gen_range(500..1000) as f32,
            _ => 1.,
        } / 100.;

        float_range(min * increase_multiplyer, max * increase_multiplyer, 2)
    }
}