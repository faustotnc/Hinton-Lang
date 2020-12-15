package org.hinton_lang.Envornment;

import java.util.HashMap;
import java.util.Map;

import org.hinton_lang.Tokens.Token;
import org.hinton_lang.Errors.RuntimeError;

public class Environment {
    /** Stores the values for this scope */
    private final Map<String, Value> values = new HashMap<>();
    /** Parent scope */
    private final Environment enclosing;

    public Environment() {
        enclosing = null;
    }

    public Environment(Environment enclosing) {
        this.enclosing = enclosing;
    }

    /**
     * Defines a variable in this scope.
     * 
     * @param name  The variable's name.
     * @param value Tha variable's value.
     */
    public void defineVar(String name, Object value) {
        values.put(name, new Value(value, Symbol.VARIABLE));
    }

    /**
     * Defines a constant in this scope.
     * 
     * @param name  The constant's name.
     * @param value The constant's value.
     */
    public void defineConst(String name, Object value) {
        values.put(name, new Value(value, Symbol.CONSTANT));
    }

    /**
     * Defines a function in this scope.
     * 
     * @param name  The function's name.
     * @param value The function's body.
     */
    public void defineFunc(String name, Object value) {
        values.put(name, new Value(value, Symbol.FUNCTION));
    }

    /**
     * Reassigns a value to an identifier in this or enclosing scopes.
     * 
     * @param name  The identifier's name.
     * @param value The new identifier's value.
     */
    public void assign(Token name, Object value) {
        if (values.containsKey(name.lexeme)) {
            if (values.get(name.lexeme).getSignature() == Symbol.CONSTANT) {
                throw new RuntimeError(name, "Cannot reassign to constant \"" + name.lexeme + "\".");
            } else {
                values.get(name.lexeme).setValue(value);
            }
            return;
        }

        // Recursively look for the given variable
        // name in enclosing environments.
        if (enclosing != null) {
            enclosing.assign(name, value);
            return;
        }

        throw new RuntimeError(name, "Undefined variable '" + name.lexeme + "'.");
    }

    /**
     * Gets a value stored by an identifier in this scope or enclosing scopes.
     * 
     * @param name The name of the identifier.
     * @return The value stored by the identifier.
     */
    public Object get(Token name) {
        if (values.containsKey(name.lexeme)) {
            Object val = values.get(name.lexeme).getValue();

            if (val == null) {
                throw new RuntimeError(name, "Variable \"" + name.lexeme + "\" has not been initialized.");
            } else {
                return val;
            }
        }

        // Recursively look for the given variable
        // name in enclosing environments.
        if (enclosing != null)
            return enclosing.get(name);

        throw new RuntimeError(name, "Undefined variable '" + name.lexeme + "'.");
    }

    /**
     * Converts the hashmap into a readable string.
     */
    public String toString() {
        StringBuilder str = new StringBuilder("====================\n");
        str.append("ENVIRONMENT\n");
        str.append("====================\n");

        int mx = 0;
        for (String n : values.keySet()) {
            if (mx < n.length()) {
                mx = n.length();
            }
        }

        for (String name : values.keySet()) {
            Object value = values.get(name).getValue();

            if (value == null)
                value = "null";

            int pad = mx - name.length() + 4;
            str.append(name).append(" ".repeat(pad)).append("= ").append(value.toString()).append("\n");
        }

        str.append("====================\n");

        return str.toString();
    }
}