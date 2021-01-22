use crate::command_api::CommandAction;
use crate::action::IAction;

#[derive(Clone)]
pub struct CASLAction {
    operation: String,
    parameters: Vec<String>,
    func: Option<&'static (dyn Fn(&Vec<String>) -> ())>,
}

impl CASLAction {
    pub fn new(conf: &CommandAction) -> CASLAction {
        if let CommandAction::CASL {operation, parameters} = conf {
            CASLAction {
                operation: operation.clone(),
                parameters: parameters.clone(),
                func: get_func_by_name(operation),
            }
        } else {panic!("Non-CASL command action given to CASLAction");}
    }
}

impl IAction for CASLAction {
    fn act(&self) {
        if let Some(f) = self.func {
            f(&self.parameters);
        }
    }
}

fn get_func_by_name(op: &str) -> Option<&'static (dyn Fn(&Vec<String>) -> ())>{
    let op_lower: &str = &op.to_lowercase();
    // this needs to be reworked
    match op_lower {
        "hello world" => Some(&hello_world),
        "debug" => Some(&print_debug),
        "warning" => Some(&print_warn),
        "error" => Some(&print_err),
        _ => None,
    }
}

// CASL action functions

fn hello_world(params: &Vec<String>) {
    if params.len() != 0 {
        println!("Hello {} world", params.get(0).unwrap());
    } else {
        println!("Hello world");
    }
}

fn print_debug(params: &Vec<String>) {
    println!("\\/ CASL DEBUG MESSAGE \\/\n{}\n/\\CASL DEBUG MESSAGE/\\", params.join("\n"));
}

fn print_warn(params: &Vec<String>) {
    println!("\\/ CASL WARNING MESSAGE \\/\n{}\n/\\CASL WARNING MESSAGE/\\", params.join("\n"));
}

fn print_err(params: &Vec<String>) {
    println!("\\/ CASL ERROR MESSAGE \\/\n{}\n/\\CASL ERROR MESSAGE/\\", params.join("\n"));
}