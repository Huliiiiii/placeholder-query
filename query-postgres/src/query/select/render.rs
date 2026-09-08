use std::{fmt::Write, num::NonZeroUsize};

use placeholder_query_core::types::ParamId;

use crate::{
    query::{
        Expr,
        expr::ExprNode,
        operator::{BinaryOp, UnaryOp},
        params::{InputScope, SqlType},
        relation::RelationNode,
    },
    statement::{Param, Statement},
};

use super::{
    ast::{Join, Order, SelectAst},
    scope::{Binding, Scope, bind_sources},
};

pub(super) fn render(ast: &SelectAst, input_scope: Option<InputScope>) -> Statement {
    let mut renderer = Renderer::new(input_scope);
    renderer.render_select(ast, None);
    Statement {
        sql: renderer.sql,
        params: renderer.params,
    }
}

struct Renderer {
    sql: String,
    params: Vec<Param>,
    placeholder_indices: Vec<Option<NonZeroUsize>>,
    next_alias: usize,
    input_scope: Option<InputScope>,
}

impl Renderer {
    fn new(input_scope: Option<InputScope>) -> Self {
        Self {
            sql: String::new(),
            params: Vec::new(),
            placeholder_indices: Vec::new(),
            next_alias: 0,
            input_scope,
        }
    }

    fn render_select(&mut self, select: &SelectAst, parent: Option<&Scope<'_>>) {
        let body = &select.body;
        let bindings = bind_sources(body, &mut self.next_alias);
        let scope = Scope {
            parent,
            bindings: &bindings,
        };

        self.sql.push_str("SELECT");
        self.render_projection(&select.projection, &scope);
        self.sql.push_str(" FROM ");
        self.render_source(bindings[0], parent);

        self.render_joins(&body.joins, &bindings, &scope, parent);

        self.render_filters(&body.filters, &scope);
        self.render_orders(&body.orders, &scope);
        self.render_limit(body.limit);
    }

    fn render_projection(&mut self, projection: &[Expr], scope: &Scope<'_>) {
        for (index, expr) in projection.iter().enumerate() {
            self.sql.push_str(if index == 0 { " " } else { ", " });
            self.render_expr(expr, scope);
        }
    }

    fn render_joins(
        &mut self,
        joins: &[Join],
        bindings: &[Binding<'_>],
        scope: &Scope<'_>,
        parent: Option<&Scope<'_>>,
    ) {
        for (index, join) in joins.iter().enumerate() {
            let right = bindings[index + 1];
            match join {
                Join::Inner { on, .. } => {
                    self.sql.push_str(" JOIN ");
                    self.render_source(right, parent);
                    self.sql.push_str(" ON ");
                    self.render_expr(on, &scope.prefix(index + 2));
                }
                Join::Lateral(_) => {
                    self.sql.push_str(" CROSS JOIN LATERAL ");
                    self.render_source(right, Some(&scope.prefix(index + 1)));
                }
            }
        }
    }

    fn render_filters(&mut self, filters: &[Expr], scope: &Scope<'_>) {
        for (index, predicate) in filters.iter().enumerate() {
            self.sql
                .push_str(if index == 0 { " WHERE " } else { " AND " });
            self.render_expr(predicate, scope);
        }
    }

    fn render_orders(&mut self, orders: &[Order], scope: &Scope<'_>) {
        for (index, order) in orders.iter().enumerate() {
            self.sql
                .push_str(if index == 0 { " ORDER BY " } else { ", " });
            self.render_expr(&order.expr, scope);
            if order.descending {
                self.sql.push_str(" DESC");
            }
        }
    }

    fn render_limit(&mut self, limit: Option<u64>) {
        if let Some(limit) = limit {
            write!(self.sql, " LIMIT {limit}").unwrap();
        }
    }

    fn render_source(&mut self, binding: Binding<'_>, parent: Option<&Scope<'_>>) {
        let alias = binding.alias;
        match &binding.relation.node {
            RelationNode::Table { table, .. } => {
                self.render_ident(table);
                write!(self.sql, " AS t{alias}").unwrap();
            }
            RelationNode::Unnest(columns) => {
                self.sql.push_str("unnest(");
                for (index, column) in columns.iter().enumerate() {
                    if index > 0 {
                        self.sql.push_str(", ");
                    }
                    self.render_param(column.param_id, &column.sql_type);
                    write!(self.sql, "::{}", column.sql_type).unwrap();
                }
                write!(self.sql, ") AS t{alias}(").unwrap();
                for (index, column) in columns.iter().enumerate() {
                    if index > 0 {
                        self.sql.push_str(", ");
                    }
                    self.render_ident(column.name);
                }
                self.sql.push(')');
            }
            RelationNode::DerivedTable(query) => {
                self.sql.push('(');
                self.render_select(query, parent);
                write!(self.sql, ") AS t{alias}").unwrap();
                if !query.projection.is_empty() {
                    self.sql.push('(');
                    for index in 0..query.projection.len() {
                        if index > 0 {
                            self.sql.push_str(", ");
                        }
                        write!(self.sql, "c{index}").unwrap();
                    }
                    self.sql.push(')');
                }
            }
        }
    }

    fn render_expr(&mut self, expr: &Expr, scope: &Scope<'_>) {
        match &*expr.0 {
            ExprNode::Column {
                relation: relation_id,
                field,
            } => {
                let binding = scope.resolve(*relation_id);
                write!(self.sql, "t{}.", binding.alias).unwrap();
                match &binding.relation.node {
                    RelationNode::Table { fields, .. } => {
                        self.render_ident(&fields[*field as usize]);
                    }
                    RelationNode::DerivedTable(_) => write!(self.sql, "c{field}").unwrap(),
                    RelationNode::Unnest(fields) => {
                        self.render_ident(fields[*field as usize].name);
                    }
                }
            }
            ExprNode::Param { param_id, sql_type } => self.render_param(*param_id, sql_type),
            ExprNode::Unary {
                op: UnaryOp::Not,
                expr,
            } => {
                self.sql.push_str("NOT (");
                self.render_expr(expr, scope);
                self.sql.push(')');
            }
            ExprNode::Binary { op, left, right } => match op {
                BinaryOp::Add => self.render_infix(" + ", left, right, scope),
                BinaryOp::Sub => self.render_infix(" - ", left, right, scope),
                BinaryOp::Eq => self.render_infix(" = ", left, right, scope),
                BinaryOp::Gt => self.render_infix(" > ", left, right, scope),
                BinaryOp::Gte => self.render_infix(" >= ", left, right, scope),
                BinaryOp::Like => self.render_infix(" LIKE ", left, right, scope),

                BinaryOp::And => self.render_logical(" AND ", left, right, scope),
                BinaryOp::Or => self.render_logical(" OR ", left, right, scope),

                BinaryOp::EqAny => {
                    self.render_quantified_comparison(" = ", "ANY", left, right, scope)
                }
                BinaryOp::EqAll => {
                    self.render_quantified_comparison(" = ", "ALL", left, right, scope)
                }
                BinaryOp::GtAny => {
                    self.render_quantified_comparison(" > ", "ANY", left, right, scope)
                }
                BinaryOp::GtAll => {
                    self.render_quantified_comparison(" > ", "ALL", left, right, scope)
                }
                BinaryOp::GteAny => {
                    self.render_quantified_comparison(" >= ", "ANY", left, right, scope)
                }
                BinaryOp::GteAll => {
                    self.render_quantified_comparison(" >= ", "ALL", left, right, scope)
                }
                BinaryOp::LikeAny => {
                    self.render_quantified_comparison(" LIKE ", "ANY", left, right, scope)
                }
                BinaryOp::LikeAll => {
                    self.render_quantified_comparison(" LIKE ", "ALL", left, right, scope)
                }
            },
        }
    }

    fn render_infix(&mut self, operator: &str, left: &Expr, right: &Expr, scope: &Scope<'_>) {
        self.render_operand(left, scope);
        self.sql.push_str(operator);
        self.render_operand(right, scope);
    }

    fn render_logical(&mut self, operator: &str, left: &Expr, right: &Expr, scope: &Scope<'_>) {
        self.sql.push('(');
        self.render_expr(left, scope);
        self.sql.push_str(operator);
        self.render_expr(right, scope);
        self.sql.push(')');
    }

    fn render_quantified_comparison(
        &mut self,
        operator: &str,
        quantifier: &str,
        left: &Expr,
        right: &Expr,
        scope: &Scope<'_>,
    ) {
        self.render_operand(left, scope);
        self.sql.push_str(operator);
        write!(self.sql, "{quantifier}(").unwrap();
        self.render_operand(right, scope);
        self.sql.push(')');
    }

    fn render_operand(&mut self, expr: &Expr, scope: &Scope<'_>) {
        match &*expr.0 {
            ExprNode::Binary { .. } | ExprNode::Unary { .. } => {
                self.sql.push('(');
                self.render_expr(expr, scope);
                self.sql.push(')');
            }
            _ => self.render_expr(expr, scope),
        }
    }

    fn render_ident(&mut self, ident: &str) {
        self.sql.push('"');
        for character in ident.chars() {
            if character == '"' {
                self.sql.push('"');
            }
            self.sql.push(character);
        }
        self.sql.push('"');
    }

    fn render_param(&mut self, param_id: ParamId, sql_type: &SqlType) {
        assert!(
            self.input_scope
                .is_some_and(|scope| scope == param_id.scope),
            "parameter refers to a different query input scope"
        );

        let input_index = usize::from(param_id.index);
        if self.placeholder_indices.len() <= input_index {
            self.placeholder_indices.resize(input_index + 1, None);
        }

        let placeholder_index = self.placeholder_indices[input_index].get_or_insert_with(|| {
            self.params.push(Param {
                index: input_index,
                ty: sql_type.clone(),
            });
            NonZeroUsize::new(self.params.len())
                .expect("inserting a parameter makes the list nonempty")
        });
        write!(self.sql, "${placeholder_index}").unwrap();
    }
}
