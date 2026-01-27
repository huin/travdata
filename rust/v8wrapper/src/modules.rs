use std::rc::Rc;

use hashbrown::HashMap;

use crate::ESScriptOrigin;

/// Definition of a single JavaScript module.
pub struct ModuleDef {
    pub src: String,
    pub origin: ESScriptOrigin,
}

pub type ModuleDefsRc = Rc<ModuleDefs>;

/// Contains a set of module definitions ready for import.
#[derive(Default)]
pub struct ModuleDefs {
    import_map: HashMap<String, ModuleDef>,
}

impl ModuleDefs {
    /// Creates a new [ModuleDefsRc] containing the given modules.
    pub fn new(import_map: HashMap<String, ModuleDef>) -> ModuleDefsRc {
        Rc::new(Self { import_map })
    }

    fn lookup<'a>(&'a self, specifier: &str) -> Option<&'a ModuleDef> {
        self.import_map.get(specifier)
    }

    /// Provides the modules in `self` to the given [v8::Context], replacing any prior
    /// [ModuleDefsRc] that were present.
    ///
    /// This allows use of [ModuleDefs::resolver_callback] with the context.
    pub fn install_into_context(self: ModuleDefsRc, context: v8::Local<'_, v8::Context>) {
        context.set_slot(self);
    }

    /// A callback for resolving modules from a [ModuleDefs] instance installed onto a
    /// [v8::Context].
    ///
    /// Typically for use as an argument to [v8::Module::instantiate_module].
    ///
    /// Must only be used as a callback from [v8], otherwise UB may occur.
    pub fn resolver_callback<'a>(
        context: v8::Local<'a, v8::Context>,
        specifier_v8: v8::Local<'a, v8::String>,
        _import_attributes: v8::Local<'a, v8::FixedArray>,
        _referrer: v8::Local<'a, v8::Module>,
    ) -> Option<v8::Local<'a, v8::Module>> {
        v8::callback_scope!(unsafe scope, context);
        let modules = match context.get_slot::<ModuleDefs>() {
            Some(modules) => modules,
            None => {
                let message = v8::String::new(
                    scope,
                    "ModuleDefs has not been installed into this v8::Context",
                )?;
                let exc = v8::Exception::error(scope, message);
                scope.throw_exception(exc);
                return None;
            }
        };

        // TODO: Look into using a non-lossy version.
        let specifier = specifier_v8.to_rust_string_lossy(scope);

        // TODO: Take time to read up on import rules in order to understand if further work is
        // needed to resolve the specifier (e.g relative to the referrer).
        let module_def = match modules.lookup(&specifier) {
            Some(module_src) => module_src,
            None => {
                let message = v8::String::new(
                    scope,
                    &format!("unknown module for specifier {:?}", specifier),
                )
                .expect("could not create message");
                let exc = v8::Exception::error(scope, message);
                scope.throw_exception(exc);
                return None;
            }
        };

        let origin_v8: v8::ScriptOrigin = module_def.origin.make_origin(scope)?;
        let module_src_v8 = v8::String::new(scope, &module_def.src)?;
        let source = &mut v8::script_compiler::Source::new(module_src_v8, Some(&origin_v8));
        let module = v8::script_compiler::compile_module(scope, source)?;

        Some(module)
    }
}
