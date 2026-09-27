use std::iter::Peekable;

use crate::errors::error::{ErrorContext, LoxError};
use crate::runtime::value::Value;
use crate::syntax::ast::*;
use crate::syntax::scanner::Scanner;
use crate::syntax::token::Kind;

pub(crate) struct Parser<'a> {
    scanner: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(scanner: Scanner<'a>) -> Parser<'a> {
        Parser {
            scanner: scanner.peekable(),
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Program, LoxError> {
        let mut stmts = Vec::new();
        loop {
            match self.scanner.peek() {
                Some(token) if token.kind() == Kind::Eof => break,
                Some(_) => stmts.push(self.parse_declaration()?),
                None => break,
            }
        }
        Ok(Program { stmts })
    }

    fn parse_declaration(&mut self) -> Result<Stmt, LoxError> {
        match self.peek_kind() {
            Some(Kind::Let) => {
                self.scanner.next();
                self.parse_let()
            }
            Some(Kind::Fun) => {
                self.scanner.next();
                self.parse_fun()
            }
            Some(Kind::Return) => {
                let token = self.scanner.next().unwrap();
                let line = token.line();
                let value = self.parse_expression()?;
                self.expect(Kind::Semicolon)?;
                Ok(Stmt::Return {
                    value,
                    line,
                    ty: None,
                })
            }
            Some(Kind::For) => {
                self.scanner.next();
                Err(LoxError::new(
                    "Feature 'for' is not yet implemented",
                    ErrorContext::Compile,
                    None,
                ))
            }
            Some(Kind::Class) => {
                self.scanner.next();
                Err(LoxError::new(
                    "Feature 'class' is not yet implemented",
                    ErrorContext::Compile,
                    None,
                ))
            }
            Some(_) => self.parse_statement(),
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, LoxError> {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Identifier => {
                let name: String = token.value().unwrap().into();
                let line = token.line();

                let initializer = match self.peek_kind() {
                    Some(Kind::Equal) => {
                        self.scanner.next();
                        Some(self.parse_expression()?)
                    }
                    Some(_) => None,
                    None => {
                        return Err(LoxError::new(
                            "Unexpected end of script",
                            ErrorContext::Compile,
                            None,
                        ));
                    }
                };
                self.expect(Kind::Semicolon)?;
                Ok(Stmt::Let {
                    name,
                    initializer,
                    line,
                    ty: None,
                })
            }
            Some(token) => Err(LoxError::new(
                &format!("unexpected {:?} #1", token),
                ErrorContext::Compile,
                None,
            )),
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn parse_fun(&mut self) -> Result<Stmt, LoxError> {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Identifier => {
                let name: String = token.value().unwrap().into();
                let line = token.line();
                let outer_err = format!("unexpected {:?} #1", token);

                self.expect(Kind::LeftParen)?;
                let params = self.parse_parameters(&outer_err)?;
                let body_stmt = self.parse_statement()?;

                let body = match body_stmt {
                    Stmt::Block { stmts, .. } => stmts,
                    other => vec![other],
                };

                Ok(Stmt::Fun {
                    name,
                    params,
                    body,
                    line,
                    ty: None,
                })
            }
            Some(token) => Err(LoxError::new(
                &format!("unexpected {:?} #1", token),
                ErrorContext::Compile,
                None,
            )),
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn parse_parameters(&mut self, outer_err: &str) -> Result<Vec<String>, LoxError> {
        let mut params = Vec::new();
        loop {
            match self.scanner.next() {
                Some(token) if token.kind() == Kind::Identifier => {
                    let inner_err = format!("unexpected {:?} #1", token);
                    let param_name: String = token.value().unwrap().into();
                    params.push(param_name);
                    match self.consume_list_separator(&inner_err)? {
                        ListSep::Break => break,
                        ListSep::Continue => continue,
                    }
                }
                Some(token) if token.kind() == Kind::RightParen => break,
                Some(_) => {
                    return Err(LoxError::new(outer_err, ErrorContext::Compile, None));
                }
                None => {
                    return Err(LoxError::new(
                        "Unexpected end of script",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            }
        }
        Ok(params)
    }

    fn parse_statement(&mut self) -> Result<Stmt, LoxError> {
        match self.peek_kind() {
            Some(Kind::If) => {
                let token = self.scanner.next().unwrap();
                self.parse_if(token.line())
            }
            Some(Kind::While) => {
                let token = self.scanner.next().unwrap();
                self.parse_while(token.line())
            }
            Some(Kind::LeftBrace) => {
                let token = self.scanner.next().unwrap();
                self.parse_block(token.line())
            }
            Some(_) => {
                let expr = self.parse_expression()?;
                let line = Self::expr_line(&expr);
                self.expect(Kind::Semicolon)?;
                Ok(Stmt::Expression {
                    expr,
                    line,
                    ty: None,
                })
            }
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn parse_block(&mut self, line: usize) -> Result<Stmt, LoxError> {
        let mut stmts = Vec::new();
        loop {
            match self.peek_kind() {
                Some(Kind::RightBrace) | Some(Kind::Eof) => break,
                Some(_) => stmts.push(self.parse_declaration()?),
                None => {
                    return Err(LoxError::new(
                        "Unexpected end of script",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            }
        }
        self.expect(Kind::RightBrace)?;
        Ok(Stmt::Block {
            stmts,
            line,
            ty: None,
        })
    }

    fn parse_if(&mut self, line: usize) -> Result<Stmt, LoxError> {
        let condition = self.parse_expression()?;
        let then_branch = Box::new(self.parse_statement()?);
        let else_branch = match self.peek_kind() {
            Some(Kind::Else) => {
                self.scanner.next();
                Some(Box::new(self.parse_statement()?))
            }
            _ => None,
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
            line,
            ty: None,
        })
    }

    fn parse_while(&mut self, line: usize) -> Result<Stmt, LoxError> {
        let condition = self.parse_expression()?;
        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::While {
            condition,
            body,
            line,
            ty: None,
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, LoxError> {
        self.parse_or(true)
    }

    fn parse_or(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        let mut expr = self.parse_and(can_assign)?;
        loop {
            match self.peek_kind() {
                Some(Kind::Or) => {
                    let token = self.scanner.next().unwrap();
                    let line = token.line();
                    let right = self.parse_and(false)?;
                    expr = Expr::Logical {
                        operator: LogicalOp::Or,
                        left: Box::new(expr),
                        right: Box::new(right),
                        line,
                        ty: None,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_and(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        let mut expr = self.parse_equality(can_assign)?;
        loop {
            match self.peek_kind() {
                Some(Kind::And) => {
                    let token = self.scanner.next().unwrap();
                    let line = token.line();
                    let right = self.parse_equality(false)?;
                    expr = Expr::Logical {
                        operator: LogicalOp::And,
                        left: Box::new(expr),
                        right: Box::new(right),
                        line,
                        ty: None,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_equality(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        let mut expr = self.parse_comparison(can_assign)?;
        loop {
            match self.peek_kind() {
                Some(Kind::EqualEqual) => {
                    let token = self.scanner.next().unwrap();
                    let line = token.line();
                    let right = self.parse_comparison(false)?;
                    expr = Expr::Binary {
                        operator: BinaryOp::Equal,
                        left: Box::new(expr),
                        right: Box::new(right),
                        line,
                        ty: None,
                    };
                }
                Some(Kind::BangEqual) => {
                    let token = self.scanner.next().unwrap();
                    let line = token.line();
                    let right = self.parse_comparison(false)?;
                    expr = Expr::Binary {
                        operator: BinaryOp::NotEqual,
                        left: Box::new(expr),
                        right: Box::new(right),
                        line,
                        ty: None,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        let mut expr = self.parse_term(can_assign)?;
        loop {
            let op = match self.peek_kind() {
                Some(Kind::Greater) => BinaryOp::Greater,
                Some(Kind::GreaterEqual) => BinaryOp::GreaterEqual,
                Some(Kind::Less) => BinaryOp::Less,
                Some(Kind::LessEqual) => BinaryOp::LessEqual,
                _ => break,
            };
            let token = self.scanner.next().unwrap();
            let line = token.line();
            let right = self.parse_term(false)?;
            expr = Expr::Binary {
                operator: op,
                left: Box::new(expr),
                right: Box::new(right),
                line,
                ty: None,
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        let mut expr = self.parse_factor(can_assign)?;
        loop {
            let op = match self.peek_kind() {
                Some(Kind::Plus) => BinaryOp::Add,
                Some(Kind::Minus) => BinaryOp::Subtract,
                Some(Kind::Concat) => BinaryOp::Concat,
                _ => break,
            };
            let token = self.scanner.next().unwrap();
            let line = token.line();
            let right = self.parse_factor(false)?;
            expr = Expr::Binary {
                operator: op,
                left: Box::new(expr),
                right: Box::new(right),
                line,
                ty: None,
            };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        let mut expr = self.parse_unary(can_assign)?;
        loop {
            let op = match self.peek_kind() {
                Some(Kind::Star) => BinaryOp::Multiply,
                Some(Kind::Slash) => BinaryOp::Divide,
                Some(Kind::Percent) => BinaryOp::Rem,
                _ => break,
            };
            let token = self.scanner.next().unwrap();
            let line = token.line();
            let right = self.parse_unary(false)?;
            expr = Expr::Binary {
                operator: op,
                left: Box::new(expr),
                right: Box::new(right),
                line,
                ty: None,
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        match self.peek_kind() {
            Some(Kind::Not) => {
                let token = self.scanner.next().unwrap();
                let line = token.line();
                let operand = self.parse_unary(false)?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Not,
                    operand: Box::new(operand),
                    line,
                    ty: None,
                })
            }
            Some(Kind::Minus) => {
                let token = self.scanner.next().unwrap();
                let line = token.line();
                let operand = self.parse_unary(false)?;
                Ok(Expr::Unary {
                    operator: UnaryOp::Negate,
                    operand: Box::new(operand),
                    line,
                    ty: None,
                })
            }
            _ => self.parse_call(can_assign),
        }
    }

    fn parse_call(&mut self, can_assign: bool) -> Result<Expr, LoxError> {
        match self.peek_kind() {
            Some(Kind::Identifier) => {
                let token = self.scanner.next().unwrap();
                let name: String = token.value().unwrap().into();
                let line = token.line();

                match self.peek_kind() {
                    Some(Kind::LeftParen) => {
                        self.scanner.next();
                        let call_err = format!("unexpected {:?} #1", token);
                        let args = self.parse_arguments(&call_err)?;
                        Ok(Expr::Call {
                            callee: name,
                            args,
                            line,
                            ty: None,
                        })
                    }
                    Some(Kind::Equal) if can_assign => {
                        self.scanner.next();
                        let value = self.parse_expression()?;
                        Ok(Expr::Assignment {
                            name,
                            value: Box::new(value),
                            line,
                            ty: None,
                        })
                    }
                    Some(Kind::Equal) => {
                        self.scanner.next();
                        Err(LoxError::new(
                            "Invalid assignment target",
                            ErrorContext::Compile,
                            None,
                        ))
                    }
                    _ => Ok(Expr::Variable {
                        name,
                        line,
                        ty: None,
                    }),
                }
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_arguments(&mut self, caller_err: &str) -> Result<Vec<Expr>, LoxError> {
        let mut args = Vec::new();
        loop {
            match self.peek_kind() {
                Some(Kind::RightParen) => {
                    self.scanner.next();
                    break;
                }
                Some(_) => {
                    args.push(self.parse_expression()?);
                    match self.consume_list_separator(caller_err)? {
                        ListSep::Break => break,
                        ListSep::Continue => continue,
                    }
                }
                None => {
                    return Err(LoxError::new(
                        "Unexpected end of script",
                        ErrorContext::Compile,
                        None,
                    ));
                }
            }
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, LoxError> {
        match self.scanner.next() {
            Some(token) if token.kind() == Kind::Nil => Ok(Expr::Literal {
                value: Value::Nil,
                line: token.line(),
                ty: None,
            }),
            Some(token) if token.kind() == Kind::Number || token.kind() == Kind::String => {
                Ok(Expr::Literal {
                    value: token.value().unwrap(),
                    line: token.line(),
                    ty: None,
                })
            }
            Some(token) if token.kind() == Kind::True => Ok(Expr::Literal {
                value: Value::Boolean(true),
                line: token.line(),
                ty: None,
            }),
            Some(token) if token.kind() == Kind::False => Ok(Expr::Literal {
                value: Value::Boolean(false),
                line: token.line(),
                ty: None,
            }),
            Some(token) if token.kind() == Kind::LeftParen => {
                let line = token.line();
                let expr = self.parse_expression()?;
                match self.peek_kind() {
                    Some(Kind::RightParen) => {
                        self.scanner.next();
                    }
                    Some(_) => {
                        return Err(LoxError::new(
                            &format!("unexpected {:?} #2", token),
                            ErrorContext::Compile,
                            None,
                        ));
                    }
                    None => {
                        return Err(LoxError::new(
                            "Unexpected end of script",
                            ErrorContext::Compile,
                            None,
                        ));
                    }
                }
                Ok(Expr::Grouping {
                    expr: Box::new(expr),
                    line,
                    ty: None,
                })
            }
            Some(token) if token.kind() == Kind::This => Err(LoxError::new(
                "Feature 'this' is not yet implemented",
                ErrorContext::Compile,
                None,
            )),
            Some(token) if token.kind() == Kind::Super => Err(LoxError::new(
                "Feature 'super' is not yet implemented",
                ErrorContext::Compile,
                None,
            )),
            Some(token) if token.kind() == Kind::Expands => Err(LoxError::new(
                "Feature 'expands' is not yet implemented",
                ErrorContext::Compile,
                None,
            )),
            Some(token) => Err(LoxError::new(
                &format!("unexpected {:?} #3", token),
                ErrorContext::Compile,
                None,
            )),
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn peek_kind(&mut self) -> Option<Kind> {
        self.scanner.peek().map(|t| t.kind())
    }

    fn expect(&mut self, kind: Kind) -> Result<(), LoxError> {
        match self.scanner.peek().cloned() {
            Some(token) if token.kind() == kind => {
                self.scanner.next();
                Ok(())
            }
            Some(token) => Err(LoxError::new(
                &format!("expected {:?}, got {:?}", kind, token),
                ErrorContext::Compile,
                None,
            )),
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
        }
    }

    fn consume_list_separator(&mut self, err_msg: &str) -> Result<ListSep, LoxError> {
        match self.peek_kind() {
            Some(Kind::Comma) => {
                self.scanner.next();
                Ok(ListSep::Continue)
            }
            Some(Kind::RightParen) => {
                self.scanner.next();
                Ok(ListSep::Break)
            }
            None => Err(LoxError::new(
                "Unexpected end of script",
                ErrorContext::Compile,
                None,
            )),
            _ => Err(LoxError::new(err_msg, ErrorContext::Compile, None)),
        }
    }

    fn expr_line(expr: &Expr) -> usize {
        match expr {
            Expr::Literal { line, .. }
            | Expr::Variable { line, .. }
            | Expr::Unary { line, .. }
            | Expr::Binary { line, .. }
            | Expr::Logical { line, .. }
            | Expr::Grouping { line, .. }
            | Expr::Call { line, .. }
            | Expr::Assignment { line, .. }
            | Expr::Narrowing { line, .. } => *line,
        }
    }
}

enum ListSep {
    Continue,
    Break,
}
