mod agent;

use agent::Agent::Agent;

fn main() {
    let new_agent = Agent::new();
    //new_agent.test();
    //new_agent.print_data();
    new_agent.dfs();
}
