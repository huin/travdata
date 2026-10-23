fn main() -> mlua_luau::Result<()> {
    exp_luau::run()
}

mod exp_luau {
    pub fn run() -> mlua_luau::Result<()> {
        let lua = mlua_luau::Lua::new();
        let print_fn = lua.create_function(|_, text: String| {
            println!("Lua printed: {text:?}");

            Ok(())
        })?;
        let globals_tb = lua.globals();
        globals_tb.set("rust_print", print_fn)?;

        // Calling a Rust function.
        let hello_chunk = lua.load(
            r#"
        local name = "World"
        rust_print("Hello "..name)
        "#,
        );
        hello_chunk.exec()?;

        print_env_table(&globals_tb)?;

        // Calling "variadic" via a single array-like table.
        let value_chunk = lua.load(
            r#"
        local arg = ...
        return arg[1]..arg[2]
        "#,
        );
        let value: String = value_chunk.call(["foo", "bar"])?;
        println!("value = {value:?}");

        // Calling variadic unpacking into local variables.
        let value_chunk2 = lua.load(
            r#"
        local a, b, c = ...
        return a + b + c
        "#,
        );
        let value2: u32 = value_chunk2.call((1, 2, 3))?;
        println!("value = {value2:?}");

        // Defining multiple functions in the environment.
        let env_table = create_environment_from_globals(&lua)?;
        lua.load(
            r#"
        rust_print("hi")
        function add(a, b)
            return a + b
        end

        function concat(a, b)
            return a .. b
        end
        "#,
        )
        .set_environment(env_table.clone())
        .exec()?;

        let add_fn = env_table.get::<mlua_luau::Function>("add")?;
        println!("add => {:?}", add_fn.call::<f32>((1, 2))?);

        luau_requirer(&lua)?;

        Ok(())
    }

    fn print_env_table(table: &mlua_luau::Table) -> mlua_luau::Result<()> {
        println!("Table has {} entries:", table.len()?);
        for pair in table.pairs::<String, mlua_luau::Value>() {
            match pair {
                Ok((k, v)) => {
                    println!(" table[{k:?}] => {v:?}");
                }
                Err(err) => {
                    println!(" table err {err:?}");
                }
            }
        }
        Ok(())
    }

    fn create_environment_from_globals(
        lua: &mlua_luau::Lua,
    ) -> mlua_luau::Result<mlua_luau::Table> {
        let env_table = lua.create_table()?;

        for pair in lua.globals().pairs::<mlua_luau::Value, mlua_luau::Value>() {
            let pair = pair?;
            env_table.set(pair.0, pair.1)?;
        }

        Ok(env_table)
    }

    pub fn luau_requirer(lua: &mlua_luau::Lua) -> mlua_luau::Result<()> {
        let require_fn = lua.create_require_function(FakeRequirer::default())?;
        let globals_tb = lua.globals();
        globals_tb.set("require", require_fn)?;

        lua.load(
            r#"
        print("./src/bar"); require("./src/bar")
        --print("../src/bar"); require("../src/bar")
        print("@foo/bar"); require("@foo/bar")
        print(require("@foo/bar"))
        "#,
        )
        .set_name("loady_chunk")
        .exec()?;

        Ok(())
    }

    #[derive(Default)]
    struct FakeRequirer {
        path: Vec<String>,
    }

    impl mlua_luau::luau::Require for FakeRequirer {
        fn is_require_allowed(&self, chunk_name: &str) -> bool {
            eprintln!("is_require_allowed {chunk_name}");
            true
        }

        fn reset(
            &mut self,
            chunk_name: &str,
        ) -> std::prelude::v1::Result<(), mlua_luau::prelude::LuaNavigateError> {
            eprintln!("reset {chunk_name}");
            self.path = vec![chunk_name.into()];
            Ok(())
        }

        fn jump_to_alias(
            &mut self,
            path: &str,
        ) -> std::prelude::v1::Result<(), mlua_luau::prelude::LuaNavigateError> {
            eprintln!("jump_to_alias {path}");
            Ok(())
        }

        fn to_parent(
            &mut self,
        ) -> std::prelude::v1::Result<(), mlua_luau::prelude::LuaNavigateError> {
            eprintln!("to_parent");
            match self.path.pop() {
                Some(_) => Ok(()),
                None => Err(mlua_luau::prelude::LuaNavigateError::NotFound),
            }
        }

        fn to_child(
            &mut self,
            name: &str,
        ) -> std::prelude::v1::Result<(), mlua_luau::prelude::LuaNavigateError> {
            eprintln!("to_child {name}");
            self.path.push(name.into());
            Ok(())
        }

        fn has_module(&self) -> bool {
            eprintln!("has_module self.path={:?}", self.path);
            self.path == ["src", "bar"]
        }

        fn cache_key(&self) -> String {
            eprintln!("cache_key");
            self.path.join("/")
        }

        fn has_config(&self) -> bool {
            eprintln!("has_config");
            true
        }

        fn config(&self) -> std::io::Result<Vec<u8>> {
            eprintln!("config");
            let config = r#"
        return {
            luau = {
                --languagemode = "nonstrict",
                --lint = {
                --    ["*"] = true,
                --    LocalUnused = false
                --},
                --linterrors = true,
                --typeerrors = true,
                --globals = {"expect"},
                aliases = {
                    foo = "./src"
                }
            }
        }
        "#;
            Ok(config.as_bytes().into())
        }

        fn loader(
            &self,
            lua: &mlua_luau::prelude::Lua,
        ) -> mlua_luau::prelude::LuaResult<mlua_luau::prelude::LuaFunction> {
            eprintln!("loader");
            if self.path == ["src", "bar"] {
                lua.create_function(|_lua, arg: mlua_luau::Value| {
                    eprintln!("arg={arg:?}");
                    Ok("loaded value")
                })
            } else {
                Err(mlua_luau::Error::RuntimeError(format!(
                    "no loader for path {:?}",
                    self.path
                )))
            }
        }
    }
}
