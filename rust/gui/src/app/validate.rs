use egui_form::{EguiValidationReport, Form, FormField, IntoFieldPath};

/// Helper trait to create a [FormField] on a [Form]. Mostly to slightly reduce noise from
/// "FormField::new".
pub trait FormFieldFactory<'f, Errors>
where
    Errors: EguiValidationReport,
{
    fn field<'a, 'c, I: IntoFieldPath<Errors::FieldPath<'c>>>(
        &'f mut self,
        into_field_path: I,
    ) -> FormField<'a, 'f, Errors>;
}

impl<'f, Errors> FormFieldFactory<'f, Errors> for Form<Errors>
where
    Errors: EguiValidationReport,
{
    fn field<'a, 'c, I: IntoFieldPath<<Errors as EguiValidationReport>::FieldPath<'c>>>(
        &'f mut self,
        into_field_path: I,
    ) -> FormField<'a, 'f, Errors> {
        FormField::new(self, into_field_path)
    }
}
