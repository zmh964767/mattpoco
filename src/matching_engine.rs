use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchWeights {
    pub skills: f64,
    pub experience: f64,
    pub education: f64,
    pub salary: f64,
    pub location: f64,
}

impl Default for MatchWeights {
    fn default() -> Self {
        Self {
            skills: 0.35,
            experience: 0.25,
            education: 0.15,
            salary: 0.15,
            location: 0.10,
        }
    }
}

impl MatchWeights {
    pub fn normalize(&self) -> Self {
        let total = self.skills + self.experience + self.education + self.salary + self.location;
        if total == 0.0 {
            return Self::default();
        }
        Self {
            skills: self.skills / total,
            experience: self.experience / total,
            education: self.education / total,
            salary: self.salary / total,
            location: self.location / total,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub skills: f64,
    pub experience: f64,
    pub education: f64,
    pub salary: f64,
    pub location: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub total_score: f64,
    pub dimensions: DimensionScore,
    pub explanation: String,
}

pub struct MatchingEngine {
    weights: MatchWeights,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            weights: MatchWeights::default(),
        }
    }

    pub fn with_weights(weights: MatchWeights) -> Self {
        Self { weights }
    }

    pub fn set_weights(&mut self, weights: MatchWeights) {
        self.weights = weights;
    }

    pub fn calculate(
        &self,
        seeker_skills: &[String],
        seeker_experience_years: f64,
        seeker_education: &str,
        seeker_expected_salary: Option<(f64, f64)>,
        seeker_locations: &[String],
        job_skills: &[String],
        job_experience_req: &str,
        job_education_req: &str,
        job_salary_range: &str,
        job_location: &str,
    ) -> MatchResult {
        let skills_score = self.score_skills(seeker_skills, job_skills);
        let experience_score = self.score_experience(seeker_experience_years, job_experience_req);
        let education_score = self.score_education(seeker_education, job_education_req);
        let salary_score = self.score_salary(seeker_expected_salary, job_salary_range);
        let location_score = self.score_location(seeker_locations, job_location);

        let normalized = self.weights.normalize();
        let total_score = (skills_score * normalized.skills
            + experience_score * normalized.experience
            + education_score * normalized.education
            + salary_score * normalized.salary
            + location_score * normalized.location)
            .min(1.0);

        let dimensions = DimensionScore {
            skills: skills_score,
            experience: experience_score,
            education: education_score,
            salary: salary_score,
            location: location_score,
        };

        let explanation = self.generate_explanation(&dimensions);

        MatchResult {
            total_score,
            dimensions,
            explanation,
        }
    }

    fn score_skills(&self, seeker_skills: &[String], job_skills: &[String]) -> f64 {
        if job_skills.is_empty() {
            return 0.5;
        }
        let seeker_lower: Vec<String> = seeker_skills.iter().map(|s| s.to_lowercase()).collect();
        let matched = job_skills
            .iter()
            .filter(|js| seeker_lower.iter().any(|ss| ss == &js.to_lowercase()))
            .count();
        matched as f64 / job_skills.len() as f64
    }

    fn score_experience(&self, seeker_years: f64, req: &str) -> f64 {
        let required_years = Self::parse_years(req);
        match required_years {
            Some(req_y) => {
                if seeker_years >= req_y {
                    1.0
                } else if seeker_years >= req_y * 0.7 {
                    0.7
                } else if seeker_years > 0.0 {
                    0.4
                } else {
                    0.1
                }
            }
            None => 0.5,
        }
    }

    fn score_education(&self, seeker_edu: &str, req: &str) -> f64 {
        let seeker_level = Self::education_level(seeker_edu);
        let req_level = Self::education_level(req);
        if req_level == 0 {
            return 0.5;
        }
        if seeker_level >= req_level {
            1.0
        } else if seeker_level == req_level - 1 {
            0.3
        } else {
            0.1
        }
    }

    fn score_salary(&self, expected: Option<(f64, f64)>, range: &str) -> f64 {
        let (expected_min, expected_max) = match expected {
            Some(e) => e,
            None => return 0.5,
        };
        let (job_min, job_max) = match Self::parse_salary_range(range) {
            Some(s) => s,
            None => return 0.5,
        };
        if expected_min > job_max {
            return 0.1;
        }
        if expected_max < job_min {
            return 0.3;
        }
        let overlap_min = expected_min.max(job_min);
        let overlap_max = expected_max.min(job_max);
        let overlap = overlap_max - overlap_min;
        let expected_range = expected_max - expected_min;
        if expected_range == 0.0 {
            return if expected_min >= job_min && expected_min <= job_max { 1.0 } else { 0.3 };
        }
        (overlap / expected_range).clamp(0.0, 1.0)
    }

    fn score_location(&self, seeker_locations: &[String], job_location: &str) -> f64 {
        if seeker_locations.is_empty() || job_location.is_empty() {
            return 0.5;
        }
        let job_lower = job_location.to_lowercase();
        let matched = seeker_locations
            .iter()
            .any(|loc| job_lower.contains(&loc.to_lowercase()) || loc.to_lowercase().contains(&job_lower));
        if matched { 1.0 } else { 0.1 }
    }

    fn parse_years(req: &str) -> Option<f64> {
        let re = regex_lite::Regex::new(r"(\d+)\s*年").ok()?;
        let cap = re.captures(req)?;
        cap[1].parse().ok()
    }

    fn education_level(edu: &str) -> u32 {
        let lower = edu.to_lowercase();
        if lower.contains("博士") || lower.contains("phd") || lower.contains("doctor") {
            5
        } else if lower.contains("硕士") || lower.contains("master") || lower.contains("研究生") {
            4
        } else if lower.contains("本科") || lower.contains("bachelor") || lower.contains("学士") {
            3
        } else if lower.contains("大专") || lower.contains("专科") || lower.contains("associate") {
            2
        } else if lower.contains("高中") || lower.contains("中专") || lower.contains("高中") {
            1
        } else {
            0
        }
    }

    fn parse_salary_range(range: &str) -> Option<(f64, f64)> {
        let re = regex_lite::Regex::new(r"(\d+)[kK万]?\s*[-–—]\s*(\d+)[kK万]?").ok()?;
        let cap = re.captures(range)?;
        let min: f64 = cap[1].parse().ok()?;
        let max: f64 = cap[2].parse().ok()?;
        if range.contains('万') {
            Some((min * 10000.0, max * 10000.0))
        } else if range.contains('k') || range.contains('K') {
            Some((min * 1000.0, max * 1000.0))
        } else {
            Some((min, max))
        }
    }

    fn generate_explanation(&self, dims: &DimensionScore) -> String {
        let mut parts = Vec::new();
        if dims.skills >= 0.8 {
            parts.push("技能匹配度高");
        } else if dims.skills >= 0.5 {
            parts.push("技能部分匹配");
        } else {
            parts.push("技能匹配度低");
        }

        if dims.experience >= 0.8 {
            parts.push("经验充足");
        } else if dims.experience >= 0.5 {
            parts.push("经验稍欠缺");
        } else {
            parts.push("经验不足");
        }

        if dims.education < 0.5 {
            parts.push("学历未达要求");
        }

        if dims.salary < 0.5 {
            parts.push("薪资期望偏高");
        }

        if dims.location < 0.5 {
            parts.push("地域不匹配");
        }

        parts.join("，")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skills_matching() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &["React".into(), "TypeScript".into(), "Node.js".into()],
            3.0, "本科", Some((15000.0, 25000.0)), &["北京".into()],
            &["React".into(), "TypeScript".into()],
            "3年", "本科", "15k-25k", "北京",
        );
        assert!((result.dimensions.skills - 1.0).abs() < 0.001);
    }

    #[test]
    fn skills_partial_match() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &["React".into(), "Vue".into()],
            3.0, "本科", Some((15000.0, 25000.0)), &["北京".into()],
            &["React".into(), "TypeScript".into(), "Python".into()],
            "3年", "本科", "15k-25k", "北京",
        );
        assert!((result.dimensions.skills - 0.333).abs() < 0.01);
    }

    #[test]
    fn experience_sufficient() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 5.0, "本科", None, &[],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!((result.dimensions.experience - 1.0).abs() < 0.001);
    }

    #[test]
    fn experience_insufficient() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 1.0, "本科", None, &[],
            &[], "5年", "本科", "15k-25k", "北京",
        );
        assert!(result.dimensions.experience < 0.5);
    }

    #[test]
    fn education_match() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 3.0, "硕士", None, &[],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!((result.dimensions.education - 1.0).abs() < 0.001);
    }

    #[test]
    fn education_below_requirement() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 3.0, "大专", None, &[],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!(result.dimensions.education < 0.5);
    }

    #[test]
    fn salary_overlap() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 3.0, "本科", Some((18000.0, 28000.0)), &["北京".into()],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!(result.dimensions.salary > 0.5);
    }

    #[test]
    fn salary_no_overlap() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 3.0, "本科", Some((30000.0, 40000.0)), &["北京".into()],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!(result.dimensions.salary < 0.3);
    }

    #[test]
    fn location_match() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 3.0, "本科", None, &["北京".into(), "上海".into()],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!((result.dimensions.location - 1.0).abs() < 0.001);
    }

    #[test]
    fn location_mismatch() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &[], 3.0, "本科", None, &["广州".into(), "深圳".into()],
            &[], "3年", "本科", "15k-25k", "北京",
        );
        assert!(result.dimensions.location < 0.5);
    }

    #[test]
    fn total_score_with_default_weights() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &["React".into(), "TypeScript".into()],
            3.0, "本科", Some((15000.0, 25000.0)), &["北京".into()],
            &["React".into(), "TypeScript".into()],
            "3年", "本科", "15k-25k", "北京",
        );
        assert!((result.total_score - 1.0).abs() < 0.001);
    }

    #[test]
    fn custom_weights() {
        let weights = MatchWeights {
            skills: 0.5,
            experience: 0.3,
            education: 0.1,
            salary: 0.05,
            location: 0.05,
        };
        let engine = MatchingEngine::with_weights(weights);
        let result = engine.calculate(
            &["React".into()],
            1.0, "本科", Some((30000.0, 40000.0)), &["广州".into()],
            &["React".into(), "Python".into()],
            "5年", "硕士", "15k-25k", "北京",
        );
        assert!(result.total_score > 0.3);
        assert!(result.dimensions.skills > result.dimensions.experience);
    }

    #[test]
    fn explanation_generated() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &["React".into()],
            1.0, "大专", Some((30000.0, 40000.0)), &["广州".into()],
            &["React".into(), "TypeScript".into(), "Python".into()],
            "5年", "本科", "15k-25k", "北京",
        );
        assert!(!result.explanation.is_empty());
        assert!(result.explanation.contains("技能"));
    }

    #[test]
    fn perfect_match() {
        let engine = MatchingEngine::new();
        let result = engine.calculate(
            &["React".into(), "TypeScript".into()],
            5.0, "硕士", Some((20000.0, 30000.0)), &["北京".into()],
            &["React".into(), "TypeScript".into()],
            "3年", "本科", "20k-30k", "北京",
        );
        assert!((result.total_score - 1.0).abs() < 0.01);
        assert!(result.explanation.contains("技能匹配度高"));
        assert!(result.explanation.contains("经验充足"));
    }
}
