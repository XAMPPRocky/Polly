# Syntax of Polly

## Elements
A element is any element starting with a "/" character. The children of an element are defined within "{}" braces. The use of braces to show hierarchy is much more succinct, than HTML end tags, and also provides a lot more versatility than being whitespace dependent.

```
/html {
    /body {
        /h1 {
            Hello World!
        }
    }
}
```

## Attributes
Attributes are defined within "()" parameters. The writer can enter either single word attributes, like "required", or "contenteditable", or key value pairings, like "style" or "href". An element with an attribute field doesn't have to also have braces. This was mainly designed for void elements such as "img", or "link", but can be for any element.

### Polly
```
/html {
    /body {
        /a(contenteditable href="index.html") {
            Hello World!
        }
        /img(src="image.jpg")
    }
}
```

### HTML


```html
<html>
    <body>
        <a contenteditable href="index.html" >
            Hello World!
        </a>
        <img src="image.jpg"/>
    </body>
</html>
```

## Classes, and ids
Since both the "class", and "id" attributes are the most commonly used attributes in HTML, they are given a syntactic sugar in a similar form to \acro{CSS} selectors. This also provides a very familiar syntax to the writer, and an easy way to write HTML selectors.

### Polly

```
  /html {
      /body {
          /h1.class.second-class#ident {
              Hello World!
          }
      }
  }
```
### HTML
```html
  <html>
      <body>
          <h1 class="class second-class" id="ident" >
              Hello World!
          </h1>
      </body>
  </html>
```

## Variables
Variables are defined with the "@" character Example: "@foo". Variables require a prefix in order to differentiate the writer's intent. The compiler will search the first level of the json for the name provided. To be able to access values that are nested within objects, the writer can use the JavaScript syntax of accessing objects Example: "@foo.bar". As Polly is purely "logic-less", you cannot define your own variables, or perform conditions on two variables. Polly will only utilize the JSON given. All declarations, and and conditions must be done beforehand, and added to the JSON file passed in.

\begin{figure}[ht!]
### Polly
```
/html {
    /body {
        Hello @name! You're from @country.region, @country.name!
    }
}
```
### JSON

```json
{
    "name": "Jane",
    "country": {
        "name": "Ireland",
        "region": "Dublin"
    }
}
```
### HTML

```html
<html>
    <body>
        Hello Jane! You're from Dublin, Ireland!
    </body>
</html>
```

## Components
Components are simply reusable blocks of markup. Components can be passed in variables, and will only read from the variables passed in. This allows them to be easily reusable, and imported into many templates, without worrying about which variables are in scope. It is also good practice to namespace your component, so you don't import it into a template, with a component that shares that name. Components can also be attached to a element, replacing the body of text.

\begin{figure}[ht!]
### Polly
```
/html {
    /body {
        /h1&component(@person.name){}
    }
}

&component(@name) {
  Hello @name!
}
```

### JSON
```json
{
  "person": {
      "name": "Joe"
  }
}
```

### HTML

```html
<html>
    <body>
        <h1> Hello Joe! </h1>
    </body>
</html>
```

## Locales
One of the key features of Polly is easy localisation. This is done using components. Polly achieves that by making use of an implied directory sturcture. So currently your Polly codebase would look like the following. In the Rust API, you can then specify which you want to render so calling `template.render("en")` would generate the English version of the website, and `template.render("de")` will render the German version, etc. Where the locales are located, or the requirement for having locales can be overwritten, if desired. The example shown below is a trivial example, but since components can be more than just text, you can have it so different locales get totally different content, or CSS rules, so you could have it in your text in English is left-aligned, where when it is in Arabic, it is right-aligned.

```
templates/
    src/
        index.polly
    locales/
        en/
            index.polly
        de/
            index.polly
        ...repeat as necessary
```

### index.poly
```
/html {
    /body {
        /h1&locales.hello-world{}
    }
}
```
### en/index.polly
```
&hello-world {
    Hello World!
}
```
### de/index.polly
```
&hello-world {
    Hallo Welt!
}
```
### English HTML
```
<html>
    <body>
        <h1> Hello World! </h1>
    </body>
</html>
```
### German HTML
```
<html>
    <body>
        <h1> Hallo Welt! </h1>
    </body>
</html>
```

## Functions
Functions are the only form of logic in Polly.  The logic of the functions themselves can only be defined in Rust. This provides the advantage of having the functions logic compiled with the program, allowing for the Rust compiler to optimise them, before they are called, instead of having polly parsing, and optimising at run-time. 

The writer can register those functions to the template, can call them from Polly. There is also a set of "standard" functions, that cover the basic logic for a templating language, such as conditionals, and iteration. Functions can be passed in components, and variables, and take in named arguments only. The writer who defines the function gets access to the full AST representation any components, and JSON. Allowing for powerful functions, that can take advantage of their context.

For example, the "std.each" function takes an array, of JSON, and a component to use to generate the html for each entry. Since we have access to both how the JSON is structured, and the Component's AST, we can have the function behave differently based on that.

What if the component had multiple arguments, and what if the JSON was an array of array's, or an array of JSON objects? With the ability to look at the AST, Polly can have so that if the array's within the array are of the same length, and the components arguments are of the same length, then it can map each entry in the array to the argument in the component.


### Polly
```
/html {
    /body {
        /ul {
            $std.each(array = @items, component = &list-item)
        }
    }
}
```
### JSON

```json
        {
            "items": ["Item One", "Item Two", "Item Three"]
        }
```
### HTML

```html
<html>
    <body>
        <ul>
            <li>Item One</li>
            <li>Item Two</li>
            <li>Item Three</li>
        </ul>
    </body>
</html>
```