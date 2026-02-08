use golem_rust::golem_wasm::wasi::clocks::wall_clock::Datetime;
use golem_rust::{agent_definition, agent_implementation};

#[agent_definition]
pub trait TestAgent {
    fn new(name: String) -> Self;

    async fn process(&mut self, v: u32) -> u32;

    fn process_async(&mut self, v: u32);

    fn process_schedule(&mut self, v: u32, delay: u32);

    fn process_self_schedule(&mut self, v: u32, delay: u32);

    fn set_calculated_value(&mut self, v: u32);

    fn get_last_input_value(&self) -> u32;

    fn get_calculated_value(&self) -> u32;
}

struct TestAgentImpl {
    _id: String,
    last_input_value: u32,
    calculated_value: u32,
}

#[agent_implementation]
impl TestAgent for TestAgentImpl {
    fn new(id: String) -> Self {
        Self {
            _id: id,
            last_input_value: 1,
            calculated_value: 1,
        }
    }

    async fn process(&mut self, v: u32) -> u32 {
        self.last_input_value = v;
        println!("processing - value: {}", v);
        let r = CalculationAgentClient::get().process(v).await;
        println!("processing - value: {}, result: {}", v, r);
        self.calculated_value = r;
        self.calculated_value
    }

    fn process_async(&mut self, v: u32) {
        self.last_input_value = v;
        println!("processing async - value: {}", v);
        CalculationAgentClient::get().trigger_process_and_notify(v, self._id.clone());
        println!("processing async - value: {} - triggered", v);
    }

    fn process_schedule(&mut self, v: u32, delay: u32) {
        self.last_input_value = v;
        let schedule_time = chrono::Utc::now() + chrono::Duration::seconds(delay as i64);
        println!(
            "post schedule - value: {} - scheduling: {}",
            v, schedule_time
        );

        CalculationAgentClient::get().schedule_process_and_notify(
            v,
            self._id.clone(),
            get_datetime(schedule_time),
        );
        println!("processing schedule - value: {} - scheduled", v);
    }

    fn process_self_schedule(&mut self, v: u32, delay: u32) {
        let schedule_time = chrono::Utc::now() + chrono::Duration::seconds(delay as i64);
        println!(
            "post self schedule - value: {} - scheduling: {}",
            v, schedule_time
        );

        TestAgentClient::get(self._id.clone()).schedule_process(v, get_datetime(schedule_time));
        println!("processing self schedule - value: {} - scheduled", v);
    }

    fn set_calculated_value(&mut self, v: u32) {
        println!("set calculated value: {}", v);
        self.calculated_value = v
    }

    fn get_last_input_value(&self) -> u32 {
        self.last_input_value
    }

    fn get_calculated_value(&self) -> u32 {
        self.calculated_value
    }
}

#[agent_definition(mode = "ephemeral")]
pub trait CalculationAgent {
    fn new() -> Self;

    fn process(&self, v: u32) -> u32;

    fn process_and_notify(&self, v: u32, id: String) -> u32;
}

struct CalculationAgentImpl {}

#[agent_implementation]
impl CalculationAgent for CalculationAgentImpl {
    fn new() -> Self {
        Self {}
    }

    fn process(&self, v: u32) -> u32 {
        v * v
    }

    fn process_and_notify(&self, v: u32, id: String) -> u32 {
        let result = v * v;

        TestAgentClient::get(id).trigger_set_calculated_value(result);
        result
    }
}

#[agent_definition(mode = "ephemeral")]
pub trait TestRequestAgent {
    fn new() -> Self;

    fn process(&mut self, id: String, v: u32);

    fn process_async(&mut self, id: String, v: u32);

    fn process_schedule(&mut self, id: String, v: u32, delay: u32);
}

struct TestRequestAgentImpl {}

#[agent_implementation]
impl TestRequestAgent for TestRequestAgentImpl {
    fn new() -> Self {
        Self {}
    }

    fn process(&mut self, id: String, v: u32) {
        println!(
            "TestRequestAgent: processing id: {}, value: {}",
            id.clone(),
            v
        );
        TestAgentClient::get(id.clone()).trigger_process(v);
        println!("TestRequestAgent: processed id: {}", id);
    }

    fn process_async(&mut self, id: String, v: u32) {
        println!(
            "TestRequestAgent: processing async id: {}, value: {}",
            id.clone(),
            v
        );
        TestAgentClient::get(id.clone()).trigger_process_async(v);
        println!(
            "TestRequestAgent: async processing triggered for id: {}",
            id
        );
    }

    fn process_schedule(&mut self, id: String, v: u32, delay: u32) {
        let schedule_time = chrono::Utc::now() + chrono::Duration::seconds(delay as i64);
        println!(
            "TestRequestAgent: scheduling process id: {}, value: {}, delay: {}, schedule: {}",
            id.clone(),
            v,
            delay,
            schedule_time
        );
        // TestAgentClient::get(id.clone()).trigger_process_schedule(v, delay);
        TestAgentClient::get(id.clone()).schedule_process_async(v, get_datetime(schedule_time));
        println!("TestRequestAgent: scheduled processing for id: {}", id);
    }
}

fn get_datetime(value: chrono::DateTime<chrono::Utc>) -> Datetime {
    let seconds = value.timestamp() as u64;
    let nanoseconds = value.timestamp_subsec_nanos();

    Datetime {
        seconds,
        nanoseconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn datetime_to_chrono(datetime: Datetime) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(datetime.seconds as i64, datetime.nanoseconds)
            .expect("Invalid datetime values")
    }

    #[test]
    fn test_get_datetime_with_current_time() {
        let now = chrono::Utc::now();
        let result = get_datetime(now);

        let converted_back = datetime_to_chrono(result);
        assert_eq!(now, converted_back);
    }
}
