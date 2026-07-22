use std::fmt::Write;

use placeholder_query_core::expr::{ColumnRef, ExprArena, ExprId, ExprNode};

use crate::utils::JoinWrite;

use crate::{
    backend::Pg,
    query::operator::{BinaryOp, UnaryOp},
    statement::PgStatement,
    value::Value,
};

use super::plan::PgSelectPlan;

impl Pg {
    pub fn build(&self, plan: &PgSelectPlan) -> PgStatement {
        render_select_plan(plan)
    }
}

pub(crate) fn render_select_plan(plan: &PgSelectPlan) -> PgStatement {
    let mut params = Vec::new();
    let mut sql = String::new();

    JoinWrite {
        buf: &mut sql,
        items: plan.select.iter().copied(),
        at_first: |sql| sql.write_str("SELECT "),
        r#do: |sql, id| {
            render_expr(sql, &plan.exprs, id, &mut params);
            Ok(())
        },
        join: |sql| sql.write_str(", "),
        at_last: |sql| write!(sql, " FROM {} AS {}", plan.from.name, plan.from.alias),
    }
    .exec()
    .unwrap();

    for join in &plan.joins {
        write!(sql, " JOIN {} AS {} ON ", join.table.name, join.table.alias).unwrap();
        render_expr(&mut sql, &plan.exprs, join.on, &mut params);
    }

    JoinWrite {
        buf: &mut sql,
        items: plan.filters.iter().copied(),
        at_first: |sql| sql.write_str(" WHERE "),
        r#do: |sql, id| {
            render_expr(sql, &plan.exprs, id, &mut params);
            Ok(())
        },
        join: |sql| sql.write_str(" AND "),
        at_last: |_| Ok(()),
    }
    .exec()
    .unwrap();

    PgStatement { sql, params }
}

fn render_expr(sql: &mut String, exprs: &ExprArena<Pg>, id: ExprId, params: &mut Vec<Value>) {
    match exprs.get(id) {
        ExprNode::Column(column_ref) => render_column(sql, column_ref),
        ExprNode::Value(value) => render_value(sql, value, params),
        ExprNode::Values(values) => render_values(sql, values, params),
        ExprNode::Unary { op, expr } => render_unary(sql, exprs, op, *expr, params),
        ExprNode::Binary { op, left, right } => {
            render_binary(sql, exprs, op, *left, *right, params)
        }
    }
}

fn render_value(sql: &mut String, value: &Value, params: &mut Vec<Value>) {
    params.push(value.clone());
    write!(sql, "${}", params.len()).unwrap();
}

fn render_values(sql: &mut String, values: &[Value], params: &mut Vec<Value>) {
    JoinWrite {
        buf: sql,
        items: values,
        at_first: |_| Ok(()),
        r#do: |sql, value| {
            render_value(sql, value, params);
            Ok(())
        },
        join: |sql| sql.write_str(", "),
        at_last: |_| Ok(()),
    }
    .exec()
    .unwrap();
}

fn render_unary(
    sql: &mut String,
    exprs: &ExprArena<Pg>,
    op: &UnaryOp,
    expr: ExprId,
    params: &mut Vec<Value>,
) {
    match op {
        UnaryOp::Not => {
            sql.write_str("NOT (").unwrap();
            render_expr(sql, exprs, expr, params);
            sql.write_str(")").unwrap();
        }
    }
}

fn render_binary(
    sql: &mut String,
    exprs: &ExprArena<Pg>,
    op: &BinaryOp,
    left: ExprId,
    right: ExprId,
    params: &mut Vec<Value>,
) {
    match op {
        BinaryOp::And => {
            sql.write_str("(").unwrap();
            render_expr(sql, exprs, left, params);
            sql.write_str(" AND ").unwrap();
            render_expr(sql, exprs, right, params);
            sql.write_str(")").unwrap();
        }
        BinaryOp::Or => {
            sql.write_str("(").unwrap();
            render_expr(sql, exprs, left, params);
            sql.write_str(" OR ").unwrap();
            render_expr(sql, exprs, right, params);
            sql.write_str(")").unwrap();
        }
        BinaryOp::Eq => {
            render_expr(sql, exprs, left, params);
            sql.write_str(" = ").unwrap();
            render_expr(sql, exprs, right, params);
        }
        BinaryOp::In => {
            if matches!(exprs.get(right), ExprNode::Values(values) if values.is_empty()) {
                sql.write_str("FALSE").unwrap();
            } else {
                render_expr(sql, exprs, left, params);
                sql.write_str(" IN (").unwrap();
                render_expr(sql, exprs, right, params);
                sql.write_str(")").unwrap();
            }
        }
        BinaryOp::Like => {
            render_expr(sql, exprs, left, params);
            sql.write_str(" LIKE ").unwrap();
            render_expr(sql, exprs, right, params);
        }
    }
}

fn render_column(sql: &mut String, column_ref: &ColumnRef) {
    match column_ref.schema() {
        Some(schema) => write!(
            sql,
            "{}.{}.{}",
            schema,
            column_ref.table_alias(),
            column_ref.name()
        )
        .unwrap(),
        None => write!(sql, "{}.{}", column_ref.table_alias(), column_ref.name()).unwrap(),
    }
}
