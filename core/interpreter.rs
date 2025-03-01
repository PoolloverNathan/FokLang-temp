use core::AST;
use core::error_handler::ErrorHandler;
use core::tokenizer::{Operator};
use std::convert::TryInto;
use core::env::Environment;

#[derive(Clone)]
pub struct Interpreter {pub error_handler:ErrorHandler}

impl Interpreter {
  fn evaluate_program(&mut self, program: AST::Node, env: &mut Environment) -> AST::Proventus {
    let mut last_eval = AST::Proventus{..Default::default()};

    match program.kind {
      AST::NodeKind::Program{body:body,id:id} => {
        env.current_node = 0;
        let mut counter = 0;
        env.node_stack = [].to_vec();
        env.push_args(body.clone());
        for i in body {
          if counter==env.current_node {
            last_eval = self.evaluate(*i, env);
            env.current_node+=1;
          }
          counter+=1;
        }
        last_eval.id = id;
      }
      _ => panic!("[Interpreter Error] Tried to evaluate non-Program Node as a Program, {:#?}", program)
    }
    return last_eval
  }
  pub fn evaluate(&mut self, node: AST::Node, env: &mut Environment) -> AST::Proventus {
    match node.kind {
      AST::NodeKind::Program{body:_,id:_} => self.evaluate_program(node, env),
      AST::NodeKind::NullLiteral{value:_} => AST::Proventus{value: AST::Fructa::Nullus, id: -1},
      AST::NodeKind::NumericLiteral{value:i} => AST::Proventus{value: AST::Fructa::Numerum(match i {AST::NodeValue::Integer(i) => i, _ => 0}), id: -1},
      AST::NodeKind::BinaryExpression{left:_,right:_,operator:_} => self.evaluate_binary_expression(node, env),
      AST::NodeKind::Identifier{symbol:_} => self.evaluate_identifier(node, env),
      AST::NodeKind::Config{arguments:_} => self.evaluate_object(node, env),
      AST::NodeKind::FunctionDeclaration{identifier: _, arguments: _, statement: _} => self.evaluate_function(node, env),
      _ => panic!("{} {:#?}", self.error_handler.interpreter("unknown_node").error_msg, node)
    }
  }
  fn evaluate_function(&mut self, node: AST::Node, env: &mut Environment) -> AST::Proventus {
    match node.kind {
      AST::NodeKind::FunctionDeclaration{identifier: identifier, arguments: arguments, statement: statement} => {
        let mut unboxed_args = Vec::<AST::Node>::new();
        for i in arguments {
          unboxed_args.push(*i);
        }
        env.declare(*identifier, AST::Proventus{value: AST::Fructa::Moenus(unboxed_args, *statement),id:-1});
        AST::Proventus{value: AST::Fructa::Nullus, id: -1}
      }
      _ => panic!("{} {:#?}", self.error_handler.interpreter("nonfunctiondeclaration_node").error_msg, node)

    }

  }
  fn evaluate_binary_expression(&mut self, node: AST::Node, env: &mut Environment) -> AST::Proventus {
    match node.kind {
      AST::NodeKind::BinaryExpression{left: node_left,right: node_right,operator: node_operator} => {
        let left = match self.evaluate(*node_left, env).value {
          AST::Fructa::Numerum(i) => i,
          _ => panic!("hi I fucked up yipee!")
        };
        let right = match self.evaluate(*node_right, env).value {
          AST::Fructa::Numerum(i) => i,
          _ => panic!("hi I fucked up big yipee!")
        };

        if self.error_handler.check_binary_expression(left,right).bool {
          panic!("[Interpreter Error] Binary Expression: {:#?}", self.error_handler.check_binary_expression(left,right).error_msg);
        }
        let result = match node_operator {
          Operator::Addition => left+right,
          Operator::Substraction => left-right,
          Operator::Multiplication => left*right,
          Operator::Division => {
            if self.error_handler.check_binary_expression_division(left,right).bool {
              panic!("[Interpreter Error] Binary Expression Division: {:#?}", 
                  self.error_handler.check_binary_expression_division(left,right).error_msg);
            }
            left/right
          },
          Operator::Exponentiation => {
            left.pow(right.try_into().unwrap())
          }
          _ => panic!("[Interpreter Error] Unknown Operator: {:#?}", node_operator)
        };

        AST::Proventus{value: AST::Fructa::Numerum(result), id: -1}
      }
      _ => panic!("[Interpreter Error] Tried to evaluate non-BinaryExpression Node as BinaryExpression, {:#?}", node)
    }
  }
  fn evaluate_identifier(&mut self, node: AST::Node, env: &mut Environment) -> AST::Proventus {
    match env.get(node).value {
      AST::Fructa::Moenus(args, statement) => {
        let mut function_env = Environment{parent: Some(Box::new(env.clone())), error_handler: self.error_handler, ..Default::default()};
        for i in 0..args.len() {
            let evaluated = self.evaluate(env.node_stack[env.current_node as usize+i+1].clone(), env);
            function_env.declare(args[i].clone(), evaluated);
            //self.evaluate();
          //get further nodes, and assing them to real arguments (eg. x = 5)
        }
        env.current_node+=args.len() as i32;
        self.evaluate(statement, &mut function_env)
        //evaluate the statement, with defined x and y

      }
      AST::Fructa::Numerum(i) => AST::Proventus {value: AST::Fructa::Numerum(i), id: -1},
      _ => panic!("damn")
    }
  }
  fn evaluate_object(&mut self, node: AST::Node, env: &mut Environment) -> AST::Proventus {
    match node.kind {
      AST::NodeKind::Config{arguments} => {
        let mut args: Vec<(AST::Node, AST::Proventus)> = vec![];
        for i in arguments {
          args.push((*i.0.clone(), self.evaluate(*i.1.clone(), env)));
        }
        AST::Proventus{value: AST::Fructa::Causor(args), id: -1}

      }
      _ => panic!("evaluation non-object as object damn")
    }
  }
}
