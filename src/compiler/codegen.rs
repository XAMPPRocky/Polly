use std::collections::BTreeMap;
use std::cell::RefCell;
use std::error;
use std::fmt;
use std::io;
use std::rc::Rc;
use std::string;

use serde_json::Value;
use super::*;
use template::Template;

const VOID_ELEMENTS: [&'static str; 13] = ["area", "base", "br", "col", "hr", "img", "input",
                                           "link", "meta", "command", "keygen", "source",
                                           "!DOCTYPE"];
pub type CodegenResult = Result<String, CodegenError>;

macro_rules! html_try {
    ($result:expr) => {
        if let Err(io_error) = $result {
            return Err(CodegenError::IoError(io_error));
        }
    }
}

pub struct Codegen {
    elements: Vec<AstResult>,
    variables: BTreeMap<String, Value>,
    parent: Rc<RefCell<Template>>,
}

impl Codegen {
    pub fn new(ast: Vec<AstResult>,
               json: BTreeMap<String, Value>,
               parent: Rc<RefCell<Template>>)
               -> Self {
        Codegen {
            elements: ast,
            variables: json,
            parent: parent,
        }
    }

    pub fn render_component(ast: Vec<AstResult>,
                            json: BTreeMap<String, Value>,
                            parent: Rc<RefCell<Template>>)
                            -> CodegenResult {
        Codegen {
            elements: ast,
            variables: json,
            parent: parent,
        }
        .generate_html()
    }

    pub fn generate_html(&mut self) -> CodegenResult {
        let mut html = String::new();

        for element in &self.elements {
            match self.render(element) {
                Ok(string) => html.push_str(&*string),
                Err(error) => return Err(error),
            }
        }
        Ok(html)
    }

    pub fn call_component(component: &Component,
                          arg_map: Option<BTreeMap<String, Value>>,
                          parent: &Rc<RefCell<Template>>)
                          -> CodegenResult {
        let mut codegen = if let Some(arg_map) = arg_map {
            Codegen {
                elements: component.ast(),
                variables: arg_map,
                parent: parent.clone(),
            }
        } else {
            Codegen {
                elements: component.ast(),
                variables: BTreeMap::new(),
                parent: parent.clone(),
            }
        };
        codegen.generate_html()
    }

    fn generate_from_component(&self, component_call: ComponentCall) -> CodegenResult {
        let parent = self.parent.borrow();
        if let Some(component) = parent.get_component(component_call.name()) {
            let args = component.args();
            let arg_values = component_call.values();
            let mut arg_map = BTreeMap::new();
            if args.len() == arg_values.len() {
                let zipped = args.iter().zip(arg_values.iter());

                for (ref arg, ref value) in zipped {

                    match **arg {
                        ArgKey::Json(ref arg_name) => {
                            if let ArgKey::Json(ref arg_value) = **value {
                                let value = match self.get_variable(arg_value) {
                                    Ok(value) => value,
                                    Err(error) => return Err(error),
                                };
                                arg_map.insert(arg_name.clone(), value);
                            }
                        }
                        ArgKey::Comp(ref name) => {
                            return Err(CodegenError::CompPassedToComp(name.clone()))
                        }
                    }
                }
                Codegen::render_component(component.ast(), arg_map, self.parent.clone())
            } else {
                Err(CodegenError::WrongNumberOfArguments(args.len(), arg_values.len()))
            }
        } else {
            Err(CodegenError::NoSuchComponent(String::from(component_call.name())))
        }
    }

    fn render_element(&self, element: &Element) -> CodegenResult {
        use std::io::Write;
        let mut html = Vec::new();
        let tag = &**element.tag();
        html_try!(write!(&mut html, "<{}", tag));

        if !element.classes().is_empty() {

            html_try!(write!(&mut html, " class=\""));
            let mut classes_iter = element.classes().iter();
            html_try!(write!(&mut html, "{}", classes_iter.next().unwrap()));

            for class in classes_iter {
                if !class.is_empty() {
                    html_try!(write!(&mut html, " {}", &*class));
                }
            }
            html_try!(write!(&mut html, "\""));
        }

        if !element.attributes().is_empty() {
            for (key, value) in element.attributes() {
                if !key.is_empty() {
                    if !value.is_empty() {
                        html_try!(write!(&mut html, " {}=\"{}\"", key, value));
                    } else {
                        html_try!(write!(&mut html, " {}", key));
                    }
                }
            }
        }

        html_try!(write!(&mut html, ">"));


        for void in &VOID_ELEMENTS {
            if void == &tag {
                return match String::from_utf8(html) {
                    Ok(html) => Ok(html),
                    Err(error) => Err(CodegenError::FromUtf8Error(error)),
                };
            }
        }

        if let Some(ref resource) = *element.resource() {
            match self.generate_from_component(resource.clone()) {
                Ok(rendered) => html_try!(write!(&mut html, "{}", rendered)),
                Err(err) => return Err(err),
            }


        } else {
            for child in element.children() {
                match self.render(child) {
                    Ok(rendered) => {
                        html_try!(write!(&mut html, "{}", rendered));
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        html_try!(write!(&mut html, "</{}>", tag));

        match String::from_utf8(html) {
            Ok(html) => Ok(html),
            Err(error) => Err(CodegenError::FromUtf8Error(error)),
        }
    }

    fn render_function(&self, function: &FunctionCall) -> CodegenResult {
        let mut arguments: BTreeMap<String, ArgValue> = BTreeMap::new();

        for (key, value) in function.args().clone() {

            match value {
                ArgKey::Json(id) => {
                    let real_value = match self.get_variable(&id) {
                        Ok(value) => value,
                        Err(error) => return Err(error),
                    };
                    arguments.insert(key, ArgValue::Json(Some(real_value)));
                }
                ArgKey::Comp(id) => {
                    let parent = self.parent.borrow();
                    let real_value = match parent.get_component(&id) {
                        Some(value) => Some(value.clone()),
                        None => None,
                    };
                    arguments.insert(key, ArgValue::Comp(real_value));
                }
            }
        }

        let parent = self.parent.borrow();
        if let Some(fun) = parent.get_function(function.identifier()) {
            match fun(arguments, &self.parent) {
                Ok(string) => Ok(string),
                Err(error) => Err(CodegenError::FunctionError(error)),
            }
        } else {
            Err(CodegenError::NoSuchFunction(String::from(function.identifier())))
        }
    }

    fn get_variable(&self, name: &String) -> Result<Value, CodegenError> {
        let segments: Vec<&str> = name.split('.').collect();

        if segments.len() == 1 {
            match self.variables.get(name) {
                Some(value) => Ok(value.clone()),
                None => Ok(Value::String(String::new())),
            }
        } else {
            match Value::Object(self.variables.clone()).find_path(&segments) {
                Some(value) => return Ok(value.clone()),
                _ => return Err(CodegenError::NotAnObjectOrNull(String::from(name.clone()))),

            }
        }
    }

    fn render(&self, token: &AstResult) -> CodegenResult {
        use super::Token::*;
        match *token {
            Ok(Html(ref element)) => self.render_element(element),
            Ok(Text(ref text)) => Ok(text.clone()),
            Ok(Variable(ref variable)) => {
                match self.get_variable(variable) {
                    Ok(value) => Ok(value_to_string(&value)),
                    Err(error) => Err(error),
                }
            }
            Ok(CompCall(ref component_call)) => {
                self.generate_from_component(component_call.clone())
            }
            Ok(Function(ref function)) => self.render_function(function),
            Err(ref error) => Err(CodegenError::AstError(error.clone())),
        }
    }
}

#[derive(Debug)]
pub enum CodegenError {
    AstError(AstError),
    CompPassedToComp(String),
    FromUtf8Error(string::FromUtf8Error),
    FunctionError(String),
    IoError(io::Error),
    NoSuchComponent(String),
    NoSuchFunction(String),
    NotAnObjectOrNull(String),
    WrongNumberOfArguments(usize, usize),
}

impl error::Error for CodegenError {
    fn description(&self) -> &str {
        use self::CodegenError::*;

        match *self {
            AstError(ref error) => error.description(),
            CompPassedToComp(_) => {
                "Currently you cannot cannot pass a component to another component: "
            }
            FromUtf8Error(ref error) => error.description(),
            FunctionError(_) => "Function produced error: ",
            IoError(ref error) => error.description(),
            NoSuchComponent(_) => "Component called doesn't exist in the current template: ",
            NoSuchFunction(_) => "Function called doesn't exist in the current template: ",
            NotAnObjectOrNull(_) => "JSON passed in wasn't an object, or was null: ",
            WrongNumberOfArguments(_, _) => "Incorrect number of arguments passed in: ",
        }
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CodegenError::*;
        use std::error::Error;

        let msg = match *self {
            AstError(ref error) => format!("{}", error),
            CompPassedToComp(ref name) => format!("{} NAME: {}", self.description(), name),
            FromUtf8Error(ref error) => format!("{}", error),
            FunctionError(ref error) => format!("{} ERROR: {}", self.description(), error),
            IoError(ref error) => format!("{}", error),
            NoSuchComponent(ref name) | NoSuchFunction(ref name) | NotAnObjectOrNull(ref name) => {
                format!("{} NAME: {}", self.description(), name)
            }
            WrongNumberOfArguments(expected, actual) => {
                format!("{} EXPECTED: {} ACTUAL: {}",
                        self.description(),
                        expected,
                        actual)
            }
        };

        write!(f, "{}", msg)
    }
}

fn value_to_string(value: &Value) -> String {
    use serde_json::Value;
    match *value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::U64(value) => value.to_string(),
        Value::F64(value) => value.to_string(),
        Value::String(ref string) => string.clone(),
        Value::Array(ref vector) => {
            let mut concated_string = String::new();
            for value in vector {
                concated_string.push_str(&*value_to_string(value));
            }
            concated_string
        }
        Value::Object(ref object) => {
            let mut concated_string = String::new();
            for ref value in object.values() {
                concated_string.push_str(&*value_to_string(*value));
            }
            concated_string
        }
    }
}
