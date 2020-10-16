// Copyright (C) 2019-2020 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::{check_tuple_type, Expression, ExpressionValue, ResolvedNode, Statement, StatementError, VariableTable};
use leo_static_check::{Attribute, ParameterType, SymbolTable, Type};
use leo_typed::{Declare, Expression as UnresolvedExpression, Span, VariableName, Variables};

use serde::{Deserialize, Serialize};

/// A `let` or `const` definition statement.
/// Defines one or more variables with resolved types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    pub declare: Declare,
    pub variables: DefinitionVariables,
    pub span: Span,
}

/// One or more variables with resolved types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefinitionVariables {
    Single(VariableName, Expression),
    Tuple(VariableName, Expression),
    MultipleVariable(Vec<VariableName>, Vec<Expression>),
    MultipleVariableTuple(Vec<VariableName>, Expression),
}

impl DefinitionVariables {
    ///
    /// Returns a new statement that defines a single variable with a single value.
    ///
    /// Performs a lookup in the given variable table if the `UnresolvedExpression` contains user-defined variables.
    ///
    fn single(
        table: &VariableTable,
        variable: VariableName,
        unresolved_expression: UnresolvedExpression,
    ) -> Result<Self, StatementError> {
        // Create a new `Expression` from the given `UnresolvedExpression`.
        let expression = Expression::new(table, unresolved_expression)?;

        Ok(DefinitionVariables::Single(variable, expression))
    }

    ///
    /// Returns a new statement that defines a tuple (single variable with multiple values).
    ///
    /// Performs a lookup in the given variable table if an `UnresolvedExpression` contains user-defined variables.
    ///
    fn tuple(
        table: &VariableTable,
        variable: VariableName,
        unresolved_expressions: Vec<UnresolvedExpression>,
    ) -> Result<Self, StatementError> {
        // Create a new tuple of `Expression`s  from the given vector of `UnresolvedExpression's.
        let tuple = Expression::tuple(table, unresolved_expressions, span.clone())?;

        Ok(DefinitionVariables::Tuple(variable, tuple))
    }

    /// Resolves multiple variables for multiple expressions
    fn multiple_variable(
        table: &mut SymbolTable,
        variables: Variables,
        expected_type: Option<Type>,
        expressions: Vec<UnresolvedExpression>,
        span: Span,
    ) -> Result<Self, StatementError> {
        // If the expected type is given, then it must be a tuple of types
        let explicit_types = check_tuple_type(expected_type, expressions.len(), span.clone())?;

        // Check number of variables == types
        if variables.names.len() != explicit_types.len() {
            return Err(StatementError::multiple_variable_types(
                variables.names.len(),
                explicit_types.len(),
                span,
            ));
        }

        // Check number of variables == expressions
        if variables.names.len() != expressions.len() {
            return Err(StatementError::multiple_variable_expressions(
                variables.names.len(),
                expressions.len(),
                span,
            ));
        }

        // Resolve expressions
        let mut expressions_resolved = vec![];

        for (expression, type_) in expressions.into_iter().zip(explicit_types) {
            let expression_resolved = Expression::resolve(table, (type_, expression))?;

            expressions_resolved.push(expression_resolved);
        }

        // Insert variables into symbol table
        for (variable, expression) in variables.names.clone().iter().zip(expressions_resolved.iter()) {
            insert_defined_variable(table, variable, expression.type_(), span.clone())?;
        }

        Ok(DefinitionVariables::MultipleVariable(
            variables.names,
            expressions_resolved,
        ))
    }

    /// Resolves multiple variables for an expression that returns a tuple
    fn multiple_variable_tuple(
        table: &mut SymbolTable,
        variables: Variables,
        expected_type: Option<Type>,
        expression: UnresolvedExpression,
        span: Span,
    ) -> Result<Self, StatementError> {
        // Resolve tuple expression
        let expression_resolved = Expression::resolve(table, (expected_type, expression.clone()))?;

        let expressions_resolved = match &expression_resolved.value {
            ExpressionValue::Tuple(expressions_resolved, _span) => expressions_resolved.clone(),
            _ => return Err(StatementError::invalid_tuple(variables.names.len(), expression, span)),
        };

        // Insert variables into symbol table
        for (variable, expression) in variables.names.clone().iter().zip(expressions_resolved.iter()) {
            insert_defined_variable(table, variable, expression.type_(), span.clone())?;
        }

        Ok(DefinitionVariables::MultipleVariableTuple(
            variables.names,
            expression_resolved,
        ))
    }
}

/// Inserts a variable definition into the given symbol table
fn insert_defined_variable(
    table: &mut SymbolTable,
    variable: &VariableName,
    type_: &Type,
    span: Span,
) -> Result<(), StatementError> {
    let attributes = if variable.mutable {
        vec![Attribute::Mutable]
    } else {
        vec![]
    };

    // Insert variable into symbol table
    let key = variable.identifier.name.clone();
    let value = ParameterType {
        identifier: variable.identifier.clone(),
        type_: type_.clone(),
        attributes,
    };

    // Check that variable name was not defined twice
    let duplicate = table.insert_name(key, value);

    if duplicate.is_some() {
        return Err(StatementError::duplicate_variable(
            variable.identifier.name.clone(),
            span,
        ));
    }

    Ok(())
}

impl Statement {
    ///
    /// Returns a new `let` or `const` definition statement from a given `UnresolvedExpression`.
    ///
    /// Performs a lookup in the given variable table if the statement contains user-defined variables.
    ///
    pub(crate) fn definition(
        table: &VariableTable,
        declare: Declare,
        variables: Variables,
        unresolved_expressions: Vec<UnresolvedExpression>,
        span: Span,
    ) -> Result<Self, StatementError> {
        let num_variables = variables.names.len();
        let num_values = unresolved_expressions.len();

        // // If an explicit type is given check that it is valid
        // let expected_type = match &variables.type_ {
        //     Some(type_) => Some(Type::new(table, type_.clone(), span.clone())?),
        //     None => None,
        // };

        let variables = if num_variables == 1 && num_values == 1 {
            // Define a single variable with a single value

            DefinitionVariables::single(
                table,
                variables.names[0].clone(),
                unresolved_expressions[0].clone(),
                span.clone(),
            )
        } else if num_variables == 1 && num_values > 1 {
            // Define a tuple (single variable with multiple values)

            DefinitionVariables::tuple(table, variables.names[0].clone(), unresolved_expressions, span.clone())
        } else if num_variables > 1 && num_values == 1 {
            // Define multiple variables for an expression that returns a tuple

            DefinitionVariables::multiple_variable_tuple(
                table,
                variables,
                unresolved_expressions[0].clone(),
                span.clone(),
            )
        } else {
            // Define multiple variables for multiple expressions

            DefinitionVariables::multiple_variable(
                table,
                variables,
                expected_type,
                unresolved_expressions,
                span.clone(),
            )
        }?;

        Ok(Statement::Definition(Definition {
            declare,
            variables,
            span,
        }))
    }
}
