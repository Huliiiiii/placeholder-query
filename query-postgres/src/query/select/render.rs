use std::fmt::Write;

use placeholder_query_core::{
    expr::{BinaryOp, ColumnRef, ExprArena, ExprId, ExprNode},
    value::Value,
};

use crate::utils::JoinWrite;

use super::plan::{PgSelectPlan, PgStatement};

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

fn render_expr(sql: &mut String, exprs: &ExprArena, id: ExprId, params: &mut Vec<Value>) {
    match exprs.get(id) {
        ExprNode::Column(column_ref) => render_column(sql, column_ref),
        ExprNode::Value(value) => {
            params.push(value.clone());
            write!(sql, "${}", params.len()).unwrap();
        }
        ExprNode::Values(values) => render_values(sql, values, params),
        ExprNode::Binary { op, left, right } => match op {
            BinaryOp::And => {
                sql.write_str("(").unwrap();
                render_expr(sql, exprs, *left, params);
                sql.write_str(" AND ").unwrap();
                render_expr(sql, exprs, *right, params);
                sql.write_str(")").unwrap();
            }
            BinaryOp::Or => {
                sql.write_str("(").unwrap();
                render_expr(sql, exprs, *left, params);
                sql.write_str(" OR ").unwrap();
                render_expr(sql, exprs, *right, params);
                sql.write_str(")").unwrap();
            }
            BinaryOp::Eq => {
                render_expr(sql, exprs, *left, params);
                sql.write_str(" = ").unwrap();
                render_expr(sql, exprs, *right, params);
            }
            BinaryOp::In => {
                render_expr(sql, exprs, *left, params);
                sql.write_str(" IN (").unwrap();
                render_expr(sql, exprs, *right, params);
                sql.write_str(")").unwrap();
            }
            BinaryOp::Like => {
                render_expr(sql, exprs, *left, params);
                sql.write_str(" LIKE ").unwrap();
                render_expr(sql, exprs, *right, params);
            }
        },
    }
}

fn render_values(sql: &mut String, values: &[Value], params: &mut Vec<Value>) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            sql.write_str(", ").unwrap();
        }

        params.push(value.clone());
        write!(sql, "${}", params.len()).unwrap();
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
