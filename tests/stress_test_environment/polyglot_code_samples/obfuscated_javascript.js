// STRESS TEST: Designed to break JavaScript parsers and tokenizers
// Unicode chaos, eval hell, prototype pollution, closure complexity

// Unicode variable names that break most tokenizers
const 𝒽𝑒𝓁𝓁𝑜 = "hello";
var ſſ = "sharp s ligature";
let 𝓪𝓫𝓬 = "mathematical alphanumeric";
const \u{1F4A9} = "pile of poo emoji"; // Valid but causes parser issues
var \u200B\u200C\u200D = "zero-width characters";
let 𖤍 = "ancient symbol";

// Extreme closure nesting (100+ levels)
function createNestedClosure(depth) {
    if (depth <= 0) return () => "base";
    
    return function() {
        const inner = createNestedClosure(depth - 1);
        return function() {
            return function() {
                return function() {
                    return function() {
                        return function() {
                            return function() {
                                return function() {
                                    return function() {
                                        return function() {
                                            return inner()();
                                        };
                                    };
                                };
                            };
                        };
                    };
                };
            };
        };
    };
}

// Proxy trap hell that breaks reflection
const proxyNightmare = new Proxy({}, {
    get(target, prop) {
        if (typeof prop === 'symbol') return undefined;
        return new Proxy(() => {}, {
            apply() {
                return proxyNightmare;
            },
            construct() {
                return proxyNightmare;
            },
            get() {
                return proxyNightmare;
            },
            set() {
                return true;
            }
        });
    },
    set(target, prop, value) {
        target[prop] = new Proxy(value, {
            get() { return proxyNightmare; }
        });
        return true;
    },
    has() { return true; },
    ownKeys() { return new Array(1000000).fill(0).map((_, i) => String(i)); },
    getOwnPropertyDescriptor() {
        return { configurable: true, enumerable: true, value: proxyNightmare };
    }
});

// Eval hell that creates code at runtime
const evilEvalCode = `
// Dynamically generated code that breaks static analysis
(() => {
    const methods = [];
    for (let i = 0; i < 10000; i++) {
        const methodName = 'method_' + i + '_' + Math.random().toString(36);
        const methodCode = \`
            function \${methodName}() {
                return function() {
                    return function() {
                        return \${i};
                    };
                };
            }
        \`;
        methods.push(methodCode);
    }
    return eval(methods.join('\\n'));
})();
`;

eval(evilEvalCode);

// Prototype pollution nightmare
Object.prototype.污染 = function() {
    return this.__proto__.__proto__.__proto__;
};

Array.prototype[Symbol.iterator] = function*() {
    while (true) yield Math.random();
};

Function.prototype.toString = () => "() => { /* obfuscated */ }";

// Generator hell with infinite sequences
function* infiniteNightmare() {
    let count = 0;
    while (true) {
        yield function*() {
            while (true) {
                yield function*() {
                    while (true) {
                        yield count++;
                    }
                }();
            }
        }();
    }
}

// Async/await chaos that breaks control flow analysis
async function asyncNightmare(depth = 0) {
    if (depth > 1000) return "deep";
    
    const promises = [];
    for (let i = 0; i < 100; i++) {
        promises.push(new Promise((resolve, reject) => {
            setTimeout(() => {
                if (Math.random() > 0.5) {
                    resolve(asyncNightmare(depth + 1));
                } else {
                    reject(new Error(`Random failure at depth ${depth}`));
                }
            }, Math.random() * 1000);
        }));
    }
    
    try {
        const results = await Promise.allSettled(promises);
        return results.map(r => r.status === 'fulfilled' ? r.value : 'failed');
    } catch (e) {
        return asyncNightmare(depth + Math.floor(Math.random() * 10));
    }
}

// WeakMap/WeakSet memory leak potential
const memoryLeakCreator = (() => {
    const weakMaps = [];
    const objects = [];
    
    return {
        createLeak() {
            const wm = new WeakMap();
            const obj = { data: new Array(100000).fill(Math.random()) };
            
            wm.set(obj, wm);
            wm.set(wm, obj);
            
            weakMaps.push(wm);
            objects.push(obj);
            
            return { wm, obj };
        }
    };
})();

// String template literal hell with nested expressions
const templateNightmare = (depth) => {
    if (depth <= 0) return "base";
    
    return `
        Depth: ${depth}
        Nested: ${templateNightmare(depth - 1)}
        Random: ${Math.random()}
        Function: ${(() => {
            const nested = `Inner template at depth ${depth}: ${
                Array.from({length: depth}, (_, i) => `
                    Nested expression ${i}: ${
                        i % 2 === 0 ? 
                            templateNightmare(Math.floor(depth / 2)) : 
                            `Depth ${depth} iteration ${i}`
                    }
                `).join('')
            }`;
            return nested;
        })()}
        Eval result: ${eval(`"Dynamic code at depth " + ${depth}`)}
    `;
};

// Class hierarchy chaos with mixins and private fields
class BaseNightmare {
    #privateField = new WeakMap();
    #anotherPrivate = Symbol('chaos');
    
    constructor() {
        this.#privateField.set(this, Math.random());
    }
    
    get [Symbol.toStringTag]() {
        return 'BaseNightmare';
    }
    
    [Symbol.iterator]() {
        return infiniteNightmare();
    }
    
    static [Symbol.hasInstance](instance) {
        return Math.random() > 0.5;
    }
}

// Mixin factory that creates dynamic inheritance chains
const createMixin = (name) => (Base) => {
    return class extends Base {
        constructor(...args) {
            super(...args);
            this[name] = new Proxy({}, {
                get: () => createMixin(name + '_nested'),
                set: () => true
            });
        }
        
        [name]() {
            return createMixin(name + '_method')(this.constructor);
        }
        
        static [name + 'Static']() {
            return createMixin(name + '_static')(BaseNightmare);
        }
    };
};

// Apply 50+ mixins to create inheritance hell
let NightmareClass = BaseNightmare;
for (let i = 0; i < 50; i++) {
    NightmareClass = createMixin(`mixin${i}`)(NightmareClass);
}

// Regular expression hell that causes catastrophic backtracking
const regexNightmare = /^(a+)+b$/; // Exponential time complexity
const attackString = 'a'.repeat(30) + 'c'; // No 'b' at end, causes backtracking

// Test the regex (this will hang most engines)
const testRegexAttack = () => {
    try {
        return regexNightmare.test(attackString);
    } catch (e) {
        return "regex failed";
    }
};

// Dynamic property creation that breaks static analysis
const dynamicPropsNightmare = {};
for (let i = 0; i < 100000; i++) {
    const propName = String.fromCharCode(...Array.from({length: 10}, () => 
        Math.floor(Math.random() * 1114111) + 1
    ));
    dynamicPropsNightmare[propName] = function() {
        return eval(`dynamicPropsNightmare["${propName}"]`);
    };
}

// Function that creates functions that create functions (meta-programming hell)
const functionFactory = (level) => {
    if (level <= 0) return () => "base function";
    
    const code = `
        (() => {
            const level${level}Fn = ${functionFactory(level - 1).toString()};
            return function generatedLevel${level}() {
                const inner = level${level}Fn();
                return function() {
                    return function() {
                        return \`Level ${level}: \${typeof inner === 'function' ? inner() : inner}\`;
                    };
                };
            };
        })()
    `;
    
    return eval(code);
};

// Buffer overflow simulation with typed arrays
const bufferNightmare = (() => {
    const ab = new ArrayBuffer(1024 * 1024); // 1MB buffer
    const views = [
        new Int8Array(ab),
        new Uint8Array(ab),
        new Int16Array(ab),
        new Uint16Array(ab),
        new Int32Array(ab),
        new Uint32Array(ab),
        new Float32Array(ab),
        new Float64Array(ab)
    ];
    
    // Attempt to access beyond buffer bounds
    views.forEach((view, i) => {
        try {
            for (let j = 0; j < view.length * 2; j++) { // Intentional overflow
                view[j] = Math.random() * Number.MAX_SAFE_INTEGER;
            }
        } catch (e) {
            console.log(`View ${i} failed:`, e.message);
        }
    });
    
    return views;
})();

// Export nightmare with circular references
const circularExport = {
    self: null,
    deep: {
        nested: {
            very: {
                deep: null
            }
        }
    },
    array: [],
    func: function() { return circularExport; }
};

circularExport.self = circularExport;
circularExport.deep.nested.very.deep = circularExport;
circularExport.array.push(circularExport);

// Main execution that triggers all nightmares
(async () => {
    try {
        console.log("Starting JavaScript nightmare...");
        
        // Trigger nested closures
        const nested = createNestedClosure(100);
        
        // Trigger proxy hell
        proxyNightmare.something.very.deeply.nested();
        
        // Trigger async nightmare
        await asyncNightmare(0);
        
        // Trigger memory leaks
        for (let i = 0; i < 1000; i++) {
            memoryLeakCreator.createLeak();
        }
        
        // Trigger template nightmare
        console.log(templateNightmare(20));
        
        // Trigger class hierarchy
        const nightmare = new NightmareClass();
        
        // Trigger regex attack (commented out to prevent hanging)
        // testRegexAttack();
        
        // Trigger dynamic functions
        const dynamicFn = functionFactory(50);
        console.log(dynamicFn()()());
        
        console.log("JavaScript nightmare completed (somehow)");
    } catch (e) {
        console.error("Nightmare failed:", e);
    }
})();

// Final code generation that creates thousands of lines
(() => {
    const code = [];
    for (let i = 0; i < 1000; i++) {
        code.push(`
            const func${i} = ${i % 2 === 0 ? 'async ' : ''}function(${
                Array.from({length: i % 10}, (_, j) => `param${j}`).join(', ')
            }) {
                ${i % 3 === 0 ? 'return ' : ''}${
                    i % 4 === 0 ? 'await ' : ''
                }func${(i + 1) % 1000}(${
                    Array.from({length: i % 5}, (_, j) => `"arg${j}"`).join(', ')
                });
            };
        `);
    }
    
    // Execute the generated code (this will break most parsers)
    eval(code.join('\n'));
})();