//! Methods to enforce constraints on expressions in a resolved Leo program.

use crate::{
    constraints::{
        new_scope_from_variable, ConstrainedCircuitMember, ConstrainedProgram, ConstrainedValue,
    },
    errors::ExpressionError,
    new_scope,
    types::{
        CircuitFieldDefinition, CircuitMember, Expression, Identifier, RangeOrExpression,
        SpreadOrExpression,
    },
    Integer, Type,
};

use snarkos_models::{
    curves::{Field, Group, PrimeField},
    gadgets::{
        r1cs::ConstraintSystem,
        utilities::{boolean::Boolean, select::CondSelectGadget},
    },
};

impl<F: Field + PrimeField, G: Group, CS: ConstraintSystem<F>> ConstrainedProgram<F, G, CS> {
    /// Enforce a variable expression by getting the resolved value
    pub(crate) fn evaluate_identifier(
        &mut self,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        unresolved_identifier: Identifier<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        // Evaluate the identifier name in the current function scope
        let variable_name = new_scope(function_scope, unresolved_identifier.to_string());
        let identifier_name = new_scope(file_scope, unresolved_identifier.to_string());

        let result_value = if let Some(value) = self.get(&variable_name) {
            // Reassigning variable to another variable
            value.clone()
        } else if let Some(value) = self.get(&identifier_name) {
            // Check global scope (function and circuit names)
            value.clone()
        } else {
            return Err(ExpressionError::UndefinedIdentifier(
                unresolved_identifier.to_string(),
            ));
        };

        match result_value {
            ConstrainedValue::Unresolved(string) => {
                Self::enforce_number_implicit(expected_types, string)
            }
            value => Ok(value),
        }
    }

    /// Enforce numerical operations
    fn enforce_add_expression(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(Self::enforce_integer_add(cs, num_1, num_2)?)
            }
            (ConstrainedValue::FieldElement(fe_1), ConstrainedValue::FieldElement(fe_2)) => {
                Ok(self.enforce_field_add(cs, fe_1, fe_2)?)
            }
            (ConstrainedValue::GroupElement(ge_1), ConstrainedValue::GroupElement(ge_2)) => {
                Ok(Self::evaluate_group_add(ge_1, ge_2))
            }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.enforce_add_expression(cs, *val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.enforce_add_expression(cs, val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.enforce_add_expression(cs, val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.enforce_add_expression(cs, val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} + {}",
                val_1, val_2,
            ))),
        }
    }

    fn enforce_sub_expression(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(Self::enforce_integer_sub(cs, num_1, num_2)?)
            }
            (ConstrainedValue::FieldElement(fe_1), ConstrainedValue::FieldElement(fe_2)) => {
                Ok(self.enforce_field_sub(cs, fe_1, fe_2)?)
            }
            (ConstrainedValue::GroupElement(ge_1), ConstrainedValue::GroupElement(ge_2)) => {
                Ok(Self::evaluate_group_sub(ge_1, ge_2))
            }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.enforce_sub_expression(cs, *val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.enforce_sub_expression(cs, val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.enforce_sub_expression(cs, val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.enforce_sub_expression(cs, val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} - {}",
                val_1, val_2,
            ))),
        }
    }

    fn enforce_mul_expression(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(Self::enforce_integer_mul(cs, num_1, num_2)?)
            }
            (ConstrainedValue::FieldElement(fe_1), ConstrainedValue::FieldElement(fe_2)) => {
                Ok(self.enforce_field_mul(cs, fe_1, fe_2)?)
            }
            // (ConstrainedValue::GroupElement(group), ConstrainedValue::FieldElement(scalar)) => {
            //     Ok(Self::evaluate_group_mul(group, scalar))
            // }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.enforce_mul_expression(cs, *val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.enforce_mul_expression(cs, val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.enforce_mul_expression(cs, val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.enforce_mul_expression(cs, val_1, val_2)
            }
            (val_1, val_2) => {
                return Err(ExpressionError::IncompatibleTypes(format!(
                    "{} * {}",
                    val_1, val_2,
                )))
            }
        }
    }

    fn enforce_div_expression(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(Self::enforce_integer_div(cs, num_1, num_2)?)
            }
            (ConstrainedValue::FieldElement(fe_1), ConstrainedValue::FieldElement(fe_2)) => {
                Ok(self.enforce_field_div(cs, fe_1, fe_2)?)
            }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.enforce_div_expression(cs, *val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.enforce_div_expression(cs, val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.enforce_div_expression(cs, val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.enforce_div_expression(cs, val_1, val_2)
            }
            (val_1, val_2) => {
                return Err(ExpressionError::IncompatibleTypes(format!(
                    "{} / {}",
                    val_1, val_2,
                )))
            }
        }
    }
    fn enforce_pow_expression(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(Self::enforce_integer_pow(cs, num_1, num_2)?)
            }
            (ConstrainedValue::FieldElement(fe_1), ConstrainedValue::Integer(num_2)) => {
                Ok(self.enforce_field_pow(cs, fe_1, num_2)?)
            }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.enforce_pow_expression(cs, *val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.enforce_pow_expression(cs, val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.enforce_pow_expression(cs, val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.enforce_pow_expression(cs, val_1, val_2)
            }
            (_, ConstrainedValue::FieldElement(num_2)) => {
                Err(ExpressionError::InvalidExponent(num_2.to_string()))
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} * {}",
                val_1, val_2,
            ))),
        }
    }

    /// Evaluate Boolean operations
    fn evaluate_eq_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Boolean(bool_1), ConstrainedValue::Boolean(bool_2)) => {
                Ok(Self::boolean_eq(bool_1, bool_2))
            }
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(Self::evaluate_integer_eq(num_1, num_2)?)
            }
            // (ResolvedValue::FieldElement(fe_1), ResolvedValue::FieldElement(fe_2)) => {
            //     Self::field_eq(fe_1, fe_2)
            // }
            (ConstrainedValue::GroupElement(ge_1), ConstrainedValue::GroupElement(ge_2)) => {
                Ok(Self::evaluate_group_eq(ge_1, ge_2))
            }
            (ConstrainedValue::Mutable(val_1), val_2) => self.evaluate_eq_expression(*val_1, val_2),
            (val_1, ConstrainedValue::Mutable(val_2)) => self.evaluate_eq_expression(val_1, *val_2),
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.evaluate_eq_expression(val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.evaluate_eq_expression(val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} == {}",
                val_1, val_2,
            ))),
        }
    }

    fn evaluate_geq_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            // (ResolvedValue::FieldElement(fe_1), ResolvedValue::FieldElement(fe_2)) => {
            //     Self::field_geq(fe_1, fe_2)
            // }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.evaluate_geq_expression(*val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.evaluate_geq_expression(val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.evaluate_geq_expression(val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.evaluate_geq_expression(val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} >= {}, values must be fields",
                val_1, val_2
            ))),
        }
    }

    fn evaluate_gt_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            // (ResolvedValue::FieldElement(fe_1), ResolvedValue::FieldElement(fe_2)) => {
            //     Self::field_gt(fe_1, fe_2)
            // }
            (ConstrainedValue::Mutable(val_1), val_2) => self.evaluate_gt_expression(*val_1, val_2),
            (val_1, ConstrainedValue::Mutable(val_2)) => self.evaluate_gt_expression(val_1, *val_2),
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.evaluate_gt_expression(val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.evaluate_gt_expression(val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} > {}, values must be fields",
                val_1, val_2
            ))),
        }
    }

    fn evaluate_leq_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            // (ResolvedValue::FieldElement(fe_1), ResolvedValue::FieldElement(fe_2)) => {
            //     Self::field_leq(fe_1, fe_2)
            // }
            (ConstrainedValue::Mutable(val_1), val_2) => {
                self.evaluate_leq_expression(*val_1, val_2)
            }
            (val_1, ConstrainedValue::Mutable(val_2)) => {
                self.evaluate_leq_expression(val_1, *val_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.evaluate_leq_expression(val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.evaluate_leq_expression(val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} <= {}, values must be fields",
                val_1, val_2
            ))),
        }
    }

    fn evaluate_lt_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            // (ResolvedValue::FieldElement(fe_1), ResolvedValue::FieldElement(fe_2)) => {
            //     Self::field_lt(fe_1, fe_2)
            // }
            (ConstrainedValue::Mutable(val_1), val_2) => self.evaluate_lt_expression(*val_1, val_2),
            (val_1, ConstrainedValue::Mutable(val_2)) => self.evaluate_lt_expression(val_1, *val_2),
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2)?;
                self.evaluate_lt_expression(val_1, val_2)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1)?;
                self.evaluate_lt_expression(val_1, val_2)
            }
            (val_1, val_2) => Err(ExpressionError::IncompatibleTypes(format!(
                "{} < {}, values must be fields",
                val_1, val_2,
            ))),
        }
    }

    /// Enforce ternary conditional expression
    fn enforce_conditional_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        first: Expression<F, G>,
        second: Expression<F, G>,
        third: Expression<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let resolved_first = match self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            vec![Type::Boolean],
            first,
        )? {
            ConstrainedValue::Boolean(resolved) => resolved,
            value => return Err(ExpressionError::IfElseConditional(value.to_string())),
        };

        let resolved_second = self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types.clone(),
            second,
        )?;
        let resolved_third =
            self.enforce_expression(cs, file_scope, function_scope, expected_types, third)?;

        match (resolved_second, resolved_third) {
            (ConstrainedValue::Boolean(bool_2), ConstrainedValue::Boolean(bool_3)) => {
                let result = Boolean::conditionally_select(cs, &resolved_first, &bool_2, &bool_3)?;
                Ok(ConstrainedValue::Boolean(result))
            }
            (ConstrainedValue::Integer(integer_2), ConstrainedValue::Integer(integer_3)) => {
                let result =
                    Integer::conditionally_select(cs, &resolved_first, &integer_2, &integer_3)?;
                Ok(ConstrainedValue::Integer(result))
            }
            (_, _) => {
                unimplemented!("conditional select gadget not implemented between given types")
            }
        }
    }

    /// Enforce array expressions
    fn enforce_array_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        mut expected_types: Vec<Type<F, G>>,
        array: Vec<Box<SpreadOrExpression<F, G>>>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        // Check explicit array type dimension if given
        let expected_dimensions = vec![];
        if !expected_types.is_empty() {
            match expected_types[0] {
                Type::Array(ref _type, ref dimensions) => {
                    expected_types = vec![expected_types[0].inner_dimension(dimensions)];
                }
                ref _type => return Err(ExpressionError::IncompatibleTypes(_type.to_string())),
            }
        }

        let mut result = vec![];
        for element in array.into_iter() {
            match *element {
                SpreadOrExpression::Spread(spread) => match spread {
                    Expression::Identifier(variable) => {
                        let array_name = new_scope_from_variable(function_scope.clone(), &variable);
                        match self.get(&array_name) {
                            Some(value) => match value {
                                ConstrainedValue::Array(array) => result.extend(array.clone()),
                                value => {
                                    return Err(ExpressionError::InvalidSpread(value.to_string()));
                                }
                            },
                            None => return Err(ExpressionError::UndefinedArray(variable.name)),
                        }
                    }
                    value => return Err(ExpressionError::InvalidSpread(value.to_string())),
                },
                SpreadOrExpression::Expression(expression) => {
                    result.push(self.enforce_expression(
                        cs,
                        file_scope.clone(),
                        function_scope.clone(),
                        expected_types.clone(),
                        expression,
                    )?);
                }
            }
        }

        // Check expected_dimensions if given
        if !expected_dimensions.is_empty() {
            if expected_dimensions[expected_dimensions.len() - 1] != result.len() {
                return Err(ExpressionError::InvalidLength(
                    expected_dimensions[expected_dimensions.len() - 1],
                    result.len(),
                ));
            }
        }

        Ok(ConstrainedValue::Array(result))
    }

    pub(crate) fn enforce_index(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        index: Expression<F, G>,
    ) -> Result<usize, ExpressionError> {
        match self.enforce_expression(cs, file_scope, function_scope, expected_types, index)? {
            ConstrainedValue::Integer(number) => Ok(number.to_usize()),
            ConstrainedValue::Unresolved(string) => Ok(string.parse::<usize>()?),
            value => Err(ExpressionError::InvalidIndex(value.to_string())),
        }
    }

    fn enforce_array_access_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        array: Box<Expression<F, G>>,
        index: RangeOrExpression<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let array = match self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types.clone(),
            *array,
        )? {
            ConstrainedValue::Array(array) => array,
            ConstrainedValue::Mutable(value) => match *value {
                ConstrainedValue::Array(array) => array,
                value => return Err(ExpressionError::InvalidArrayAccess(value.to_string())),
            },
            value => return Err(ExpressionError::InvalidArrayAccess(value.to_string())),
        };

        match index {
            RangeOrExpression::Range(from, to) => {
                let from_resolved = match from {
                    Some(from_index) => from_index.to_usize(),
                    None => 0usize, // Array slice starts at index 0
                };
                let to_resolved = match to {
                    Some(to_index) => to_index.to_usize(),
                    None => array.len(), // Array slice ends at array length
                };
                Ok(ConstrainedValue::Array(
                    array[from_resolved..to_resolved].to_owned(),
                ))
            }
            RangeOrExpression::Expression(index) => {
                let index_resolved =
                    self.enforce_index(cs, file_scope, function_scope, expected_types, index)?;
                Ok(array[index_resolved].to_owned())
            }
        }
    }

    fn enforce_circuit_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        identifier: Identifier<F, G>,
        members: Vec<CircuitFieldDefinition<F, G>>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let mut program_identifier = new_scope(file_scope.clone(), identifier.to_string());

        if identifier.is_self() {
            program_identifier = file_scope.clone();
        }

        if let Some(ConstrainedValue::CircuitDefinition(circuit_definition)) =
            self.get_mut(&program_identifier)
        {
            let circuit_identifier = circuit_definition.identifier.clone();
            let mut resolved_members = vec![];
            for member in circuit_definition.members.clone().into_iter() {
                match member {
                    CircuitMember::CircuitField(identifier, _type) => {
                        let matched_field = members
                            .clone()
                            .into_iter()
                            .find(|field| field.identifier.eq(&identifier));
                        match matched_field {
                            Some(field) => {
                                // Resolve and enforce circuit object
                                let field_value = self.enforce_expression(
                                    cs,
                                    file_scope.clone(),
                                    function_scope.clone(),
                                    vec![_type.clone()],
                                    field.expression,
                                )?;

                                resolved_members
                                    .push(ConstrainedCircuitMember(identifier, field_value))
                            }
                            None => {
                                return Err(ExpressionError::ExpectedCircuitValue(
                                    identifier.to_string(),
                                ))
                            }
                        }
                    }
                    CircuitMember::CircuitFunction(_static, function) => {
                        let identifier = function.function_name.clone();
                        let mut constrained_function_value =
                            ConstrainedValue::Function(Some(circuit_identifier.clone()), function);

                        if _static {
                            constrained_function_value =
                                ConstrainedValue::Static(Box::new(constrained_function_value));
                        }

                        resolved_members.push(ConstrainedCircuitMember(
                            identifier,
                            constrained_function_value,
                        ));
                    }
                };
            }

            Ok(ConstrainedValue::CircuitExpression(
                circuit_identifier.clone(),
                resolved_members,
            ))
        } else {
            Err(ExpressionError::UndefinedCircuit(identifier.to_string()))
        }
    }

    fn enforce_circuit_access_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        circuit_identifier: Box<Expression<F, G>>,
        circuit_member: Identifier<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let (circuit_name, members) = match self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types.clone(),
            *circuit_identifier.clone(),
        )? {
            ConstrainedValue::CircuitExpression(name, members) => (name, members),
            ConstrainedValue::Mutable(value) => match *value {
                ConstrainedValue::CircuitExpression(name, members) => (name, members),
                value => return Err(ExpressionError::InvalidCircuitAccess(value.to_string())),
            },
            value => return Err(ExpressionError::InvalidCircuitAccess(value.to_string())),
        };

        let matched_member = members
            .clone()
            .into_iter()
            .find(|member| member.0 == circuit_member);

        match matched_member {
            Some(member) => {
                match &member.1 {
                    ConstrainedValue::Function(ref _circuit_identifier, ref _function) => {
                        // Pass static circuit fields into function call by value
                        for stored_member in members {
                            match &stored_member.1 {
                                ConstrainedValue::Function(_, _) => {}
                                ConstrainedValue::Static(_) => {}
                                _ => {
                                    let circuit_scope =
                                        new_scope(file_scope.clone(), circuit_name.to_string());
                                    let function_scope =
                                        new_scope(circuit_scope, member.0.to_string());
                                    let field =
                                        new_scope(function_scope, stored_member.0.to_string());

                                    self.store(field, stored_member.1.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
                Ok(member.1)
            }
            None => Err(ExpressionError::UndefinedCircuitObject(
                circuit_member.to_string(),
            )),
        }
    }

    fn enforce_circuit_static_access_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        circuit_identifier: Box<Expression<F, G>>,
        circuit_member: Identifier<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        // Get defined circuit
        let circuit = match self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types.clone(),
            *circuit_identifier.clone(),
        )? {
            ConstrainedValue::CircuitDefinition(circuit_definition) => circuit_definition,
            value => return Err(ExpressionError::InvalidCircuitAccess(value.to_string())),
        };

        // Find static circuit function
        let matched_function = circuit.members.into_iter().find(|member| match member {
            CircuitMember::CircuitFunction(_static, _function) => *_static,
            _ => false,
        });

        // Return errors if no static function exists
        let function = match matched_function {
            Some(CircuitMember::CircuitFunction(_static, function)) => {
                if _static {
                    function
                } else {
                    return Err(ExpressionError::InvalidStaticFunction(
                        function.function_name.to_string(),
                    ));
                }
            }
            _ => {
                return Err(ExpressionError::UndefinedStaticFunction(
                    circuit.identifier.to_string(),
                    circuit_member.to_string(),
                ))
            }
        };

        Ok(ConstrainedValue::Function(
            Some(circuit.identifier),
            function,
        ))
    }

    fn enforce_function_call_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        function: Box<Expression<F, G>>,
        arguments: Vec<Expression<F, G>>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let function_value = self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types.clone(),
            *function.clone(),
        )?;

        let (outer_scope, function_call) = match function_value {
            ConstrainedValue::Function(circuit_identifier, function) => {
                let mut outer_scope = file_scope.clone();
                // If this is a circuit function, evaluate inside the circuit scope
                if circuit_identifier.is_some() {
                    outer_scope = new_scope(file_scope, circuit_identifier.unwrap().to_string());
                }

                (outer_scope, function.clone())
            }
            value => return Err(ExpressionError::UndefinedFunction(value.to_string())),
        };

        match self.enforce_function(cs, outer_scope, function_scope, function_call, arguments) {
            Ok(ConstrainedValue::Return(return_values)) => {
                if return_values.len() == 1 {
                    Ok(return_values[0].clone())
                } else {
                    Ok(ConstrainedValue::Return(return_values))
                }
            }
            Ok(_) => Err(ExpressionError::FunctionDidNotReturn(function.to_string())),
            Err(error) => Err(ExpressionError::from(Box::new(error))),
        }
    }

    pub(crate) fn enforce_number_implicit(
        expected_types: Vec<Type<F, G>>,
        value: String,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        if expected_types.len() == 1 {
            return Ok(ConstrainedValue::from_type(value, &expected_types[0])?);
        }

        Ok(ConstrainedValue::Unresolved(value))
    }

    pub(crate) fn enforce_expression(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: Vec<Type<F, G>>,
        expression: Expression<F, G>,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match expression {
            // Variables
            Expression::Identifier(unresolved_variable) => self.evaluate_identifier(
                file_scope,
                function_scope,
                expected_types,
                unresolved_variable,
            ),

            // Values
            Expression::Integer(integer) => Ok(Self::get_integer_constant(integer)),
            Expression::FieldElement(fe) => Ok(Self::get_field_element_constant(fe)),
            Expression::GroupElement(gr) => Ok(ConstrainedValue::GroupElement(gr)),
            Expression::Boolean(bool) => Ok(Self::get_boolean_constant(bool)),
            Expression::Implicit(value) => Self::enforce_number_implicit(expected_types, value),

            // Binary operations
            Expression::Add(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                self.enforce_add_expression(cs, resolved_left, resolved_right)
            }
            Expression::Sub(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                self.enforce_sub_expression(cs, resolved_left, resolved_right)
            }
            Expression::Mul(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                self.enforce_mul_expression(cs, resolved_left, resolved_right)
            }
            Expression::Div(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                self.enforce_div_expression(cs, resolved_left, resolved_right)
            }
            Expression::Pow(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                self.enforce_pow_expression(cs, resolved_left, resolved_right)
            }

            // Boolean operations
            Expression::Not(expression) => Ok(Self::evaluate_not(self.enforce_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                *expression,
            )?)?),
            Expression::Or(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.enforce_or(cs, resolved_left, resolved_right)?)
            }
            Expression::And(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.enforce_and(cs, resolved_left, resolved_right)?)
            }
            Expression::Eq(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.evaluate_eq_expression(resolved_left, resolved_right)?)
            }
            Expression::Geq(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.evaluate_geq_expression(resolved_left, resolved_right)?)
            }
            Expression::Gt(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.evaluate_gt_expression(resolved_left, resolved_right)?)
            }
            Expression::Leq(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.evaluate_leq_expression(resolved_left, resolved_right)?)
            }
            Expression::Lt(left, right) => {
                let resolved_left = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types.clone(),
                    *left,
                )?;
                let resolved_right = self.enforce_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *right,
                )?;

                Ok(self.evaluate_lt_expression(resolved_left, resolved_right)?)
            }

            // Conditionals
            Expression::IfElse(first, second, third) => self.enforce_conditional_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                *first,
                *second,
                *third,
            ),

            // Arrays
            Expression::Array(array) => {
                self.enforce_array_expression(cs, file_scope, function_scope, expected_types, array)
            }
            Expression::ArrayAccess(array, index) => self.enforce_array_access_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                array,
                *index,
            ),

            // Circuits
            Expression::Circuit(circuit_name, members) => self.enforce_circuit_expression(
                cs,
                file_scope,
                function_scope,
                circuit_name,
                members,
            ),
            Expression::CircuitMemberAccess(circuit_variable, circuit_member) => self
                .enforce_circuit_access_expression(
                    cs,
                    file_scope,
                    function_scope,
                    expected_types,
                    circuit_variable,
                    circuit_member,
                ),
            Expression::CircuitStaticFunctionAccess(circuit_identifier, circuit_member) => self
                .enforce_circuit_static_access_expression(
                    cs,
                    file_scope,
                    function_scope,
                    expected_types,
                    circuit_identifier,
                    circuit_member,
                ),

            // Functions
            Expression::FunctionCall(function, arguments) => self.enforce_function_call_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                function,
                arguments,
            ),
        }
    }
}