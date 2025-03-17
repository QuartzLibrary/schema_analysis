/*!
This module holds the analysis logic and an overview of how it works.

# All-of-the-Details

Let's ~~quickly~~ run through how Serde works as that's pretty much all there is to it.

## Simple Serde:

**`Program`**: Hey there `Type`, we need to get you deserialized, here is the [Deserializer] you need to districate yourself from.

```ignore
// Program uses:
let _: Type = <Type as Deserialize>::deserialize(deserializer)
```

**`Type`**: Ok!

**`Type`**: [Visitor], we have reviewed what I need, are you ready?

**`Visitor`**: I am!

**`Type`**: Dear [Deserializer], let me tell you *all* about how I want you to deserialize me by sending my ambassador and choosing this very specific method.

```ignore
// Type uses:
let _: Visitor::Value = deserializer.deserialize_str(Visitor::default())
```

**`Deserializer`**: Fancy seeing you here, [Visitor], welcome in my [str] department, let me get it for you right away.

```ignore
// Deserializer uses:
let _: &str = arcane_magic(input);
```

**`Deserializer`**: here we go [Visitor], a nice [str] for you.

```ignore
// Deserializer uses:
let _: Visitor::Value = visitor.visit_str(nice_str);
```

**`Visitor`**: Many thanks [Deserializer], I'll take it over from here.

> [Visitor] dramatically breaks forth wall.
>> _Alas, the world is cruel and this data format stores its boolean values as strings **[crowd gasps]**, but worry not as I will now parse this abomination into a proper type._

```ignore
// Visitor uses:
let _: bool = purification_magic(abomination);
```

**`Visitor`**: Here we go [Deserializer], this is the actual value the `Type` wants.

**`Deserializer`**: Thank you, I have no idea what it is, but I'll pass it over.

**`Deserializer`**: Here you go `Type`, here is a value of type [Visitor::Value] as requested.

**`Type`**: Oh, such a nice and shiny [bool], this is all the information I need to finish building myself. Thank you very much, and say bye to [Visitor] for me!

### **Result**:
The `Type` defined what the serialized form is supposed to be by calling the correct method on the [Deserializer], then the [Visitor] handled the conversion before the value was returned to the `Type`.
Why not return directly and let `Type` do the conversion? To allow for more use cases like the one below.

---

## Less Simple Serde

> Or: ~~`Type`'s serialized sibling has a multiple personality disorder~~

`[...]`

**`Type`**: Dear [Deserializer], let me tell you ~~*all*~~ *some* about how I want you to deserialize me by sending my ambassador. And that's it. You only get my ambassador. No, I'm not gonna conveniently call the correct method so that you know what to deserialize next. What? This works only on self describing formats? Do I look like I care?

```ignore
// Type uses:
let _: Visitor::Value = deserializer.deserialize_any(Visitor::default())
// It's super effective.
```

**`Deserializer`**: Fancy seeing you here, [Visitor], welcome in my `any` department, where all is possible.

```ignore
// Deserializer uses:
let _: SerdeType = deduce_and_extract_with_arcane_magic(input);
```

> ***NDR***: `SerdeType` may be any type supported by the serde [Visitor] interface, like [bool], [i32], &str, &'de str, maps, sequences... This will work on any of those that the `Visitor` has implemented.

**`Deserializer`**: here we go [Visitor], a nice `SerdeType` for you.

```ignore
// Deserializer uses:
let _: Visitor::Value = visitor.visit_serde_type(nice_serde_type);
```

**`Visitor`**: Many thanks [Deserializer], I'll take it over from here.

> [Visitor] dramatically breaks forth wall.
>> _Alas, the world is cruel and `Type`'s serialized sibling suffers from a severe case of MPD. Do not despair, however, for [Deserializer] told me it found a `SerdeType` which I'll make sure to transfigure into a [bool]._

```ignore
// Visitor uses:
let _: bool = transfigure(serde_type);
```

**`Visitor`**: Here we go [Deserializer], this is the actual value the `Type` wants.

**`Deserializer`**: Thank you, I have no idea what it is, but I'll pass it over.

**`Deserializer`**: Here you go `Type`, here is a value of type [Visitor::Value] as requested.

**`Type`**: Oh, such a nice and shiny [bool], this is all the information I need to finish building myself. Thank you very much, and say bye to [Visitor] for me!

### **Result**:
The `Type` did not give any hint as to what it wants except by passing the [Visitor], so the [Deserializer] was forced to figure it out and then pass what it found to the [Visitor] which held the knowledge on how possibly multiple types are converted into [Visitor::Value]s. This allows the `Type` to be deserialized from different physical types as long as the format itself is self-describing. More complex setups are also possible.


---

## Come on, just tell me how it works already!

I already did! It's the second option up there.

The schema analysis is done by the [Deserializer], that when [Deserializer::deserialize_any] is called uses its understanding of the format to decide what's next.
Then it hands off the value to the [Visitor] which returns a [Schema] enum value depending on the type found.

This means that the analysis is implemented by three distinct pieces:
- The [Schema] enum, which represents the available Serde values.
    - The [Schema] also holds various [context](crate::context) objects that help keep track of what kind of values the schema has seen.
- The [InferredSchema] which merely wraps around [Schema] so that the [Schema] itself may also implement the normal version of [Deserialize] for storage.
- A [Visitor] implementation which behaves as described above.

---

## Detailed Details

There is bit more to the story: since all the code above is defined at compile time and the `Program` 'talks' to the `Type`, not the actual `TypeValue`, we can only 'create new values' out of ~~thin air~~ the input, not use existing values during the deserialization process.
This would stop us from using an already inferred schema to expand as that is a runtime value.

Unsurprisingly at this point, Serde covers this use case too by introducing a different trait [DeserializeSeed]. [DeserializeSeed] is essentially equivalent to [Deserialize], with the difference that it also passes along the value it is being called on ([Deserialize::deserialize]​(deserializer)/[DeserializeSeed::deserialize]​(self, deserializer)).
This means that the value you call [DeserializeSeed::deserialize] on is available to both the deserialize call and inside the [Visitor] (if you put the value there). In our case we hide the [Schema] as a mutable reference inside the [Visitor], so that when the [Visitor] is called by the [Deserializer] it'll be able to modify the [Schema] with additional juicy details.

A mandatory illustration follows:

**`Program`**: Hey there `TypeValue`, we need your help deserializing this `TargetValue`, here is the [Deserializer] you need to districate it from.

```ignore
// Program uses:
let original: Type = Default::default();
let target_value: <Type as DeserializeSeed>::Value = original.deserialize(deserializer);
// Both deserializer and original are moved in the function.
```

**`TypeValue`**: Ok!

**`TypeValue`**: [Visitor], here is the runtime-recipe only I know that is needed to deserialize `TargetValue`, are you ready?

**`Visitor`**: I am!

**`TypeValue`**: Dear [Deserializer], let me tell you *all* about how I want you to deserialize me by sending my ambassador and choosing this very specific method.

```ignore
// Type uses:
let visitor: Visitor = Visitor::with_recipe(self.recipe);
let _: Visitor::Value = deserializer.deserialize_str(visitor);
```

`[...]`

*/
use serde::{de::DeserializeSeed, Deserialize, Deserializer};

#[allow(unused_imports)]
use serde::de::Visitor; // For docs above.

use crate::{
    context::{Context, DefaultContext},
    Coalesce, Schema,
};

mod field;
mod schema;
mod schema_seed;

use schema::SchemaVisitor;
use schema_seed::SchemaVisitorSeed;

/**
[InferredSchema] is at the heart of this crate, it is a wrapper around [Schema] that interfaces
with the analysis code.
It implements both [Deserialize] and [DeserializeSeed] to allow for analysis both when no schema is
yet available and when we wish to expand an existing schema (for data across files, for example).
 */
pub struct InferredSchema<C: Context = DefaultContext> {
    /// Where the juicy info lays.
    pub schema: Schema<C>,
}
impl<C: Context> Coalesce for InferredSchema<C>
where
    Schema<C>: Coalesce,
{
    fn coalesce(&mut self, other: Self) {
        self.schema.coalesce(other.schema)
    }
}
// (no schema + no context) -> (schema + no context)
impl<'de, C: Context + Default> Deserialize<'de> for InferredSchema<C> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let context = C::default();
        let visitor = SchemaVisitor { context: &context };
        let schema = deserializer.deserialize_any(visitor)?;
        Ok(InferredSchema { schema })
    }
}
// (schema + no context) -> (schema + no context)
impl<'de, C: Context + Default> DeserializeSeed<'de> for &mut InferredSchema<C>
where
    Schema<C>: Coalesce,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let context = C::default();
        let visitor = SchemaVisitorSeed {
            context: &context,
            schema: &mut self.schema,
        };
        deserializer.deserialize_any(visitor)?;
        Ok(())
    }
}

/**
[InferredSchemaWithContext] is an experimental feature that allows the user to provide a custom
context.

It is meant to be used along with [Aggregators](crate::context::Aggregators) holding
custom aggregators as trait objects.
To use it, construct a [Default] [Context] and push custom aggregators to the `other_aggregators`
fields present on some sub-contexts like [StringContext](crate::context::StringContext). The
custom aggregator will need to implement [CoalescingAggregator](crate::traits::CoalescingAggregator).
 */
pub struct InferredSchemaWithContext<C: Context> {
    /// The schema holds the actual description of the data.
    pub schema: Schema<C>,
    /// The context may be user-provided with additional aggregators.
    pub context: C,
}
impl<C: Context> Coalesce for InferredSchemaWithContext<C>
where
    Schema<C>: Coalesce,
{
    fn coalesce(&mut self, other: Self)
    where
        Self: Sized,
    {
        self.schema.coalesce(other.schema);
    }
}
// (schema + context) -> (schema + context)
impl<'de, C: Context> DeserializeSeed<'de> for &mut InferredSchemaWithContext<C>
where
    Schema<C>: Coalesce,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = SchemaVisitorSeed {
            context: &self.context,
            schema: &mut self.schema,
        };
        deserializer.deserialize_any(visitor)?;
        Ok(())
    }
}
// (no schema + context) -> (schema + context)
impl<C: Context> InferredSchemaWithContext<C> {
    /// Deserialization of a new schema using a context, returns a [InferredSchemaWithContext] that
    /// can be used to deserialize further files and reuse the context.
    pub fn deserialize_schema<'de, D>(context: C, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = SchemaVisitor { context: &context };
        let schema = deserializer.deserialize_any(visitor)?;
        Ok(InferredSchemaWithContext { context, schema })
    }
}

mod boilerplate {
    use std::fmt;

    use crate::{context::Context, Schema};

    use super::{InferredSchema, InferredSchemaWithContext};

    // Auto-generated, with bounds changed. (TODO: use perfect derive.)
    impl<C: Context> fmt::Debug for InferredSchema<C>
    where
        Schema<C>: fmt::Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("InferredSchema")
                .field("schema", &self.schema)
                .finish()
        }
    }
    // Auto-generated, with bounds changed. (TODO: use perfect derive.)
    impl<C: Context> Clone for InferredSchema<C>
    where
        Schema<C>: Clone,
    {
        fn clone(&self) -> Self {
            Self {
                schema: self.schema.clone(),
            }
        }
    }
    // Auto-generated, with bounds changed. (TODO: use perfect derive.)
    impl<C: Context> PartialEq for InferredSchema<C>
    where
        Schema<C>: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.schema == other.schema
        }
    }

    // Auto-generated, with bounds changed. (TODO: use perfect derive.)
    impl<C: Context + fmt::Debug> fmt::Debug for InferredSchemaWithContext<C>
    where
        Schema<C>: fmt::Debug,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("InferredSchemaWithContext")
                .field("schema", &self.schema)
                .field("context", &self.context)
                .finish()
        }
    }
    // Auto-generated, with bounds changed. (TODO: use perfect derive.)
    impl<C: Context> Clone for InferredSchemaWithContext<C>
    where
        Schema<C>: Clone,
        C: Clone,
    {
        fn clone(&self) -> Self {
            Self {
                schema: self.schema.clone(),
                context: self.context.clone(),
            }
        }
    }
    // Auto-generated, with bounds changed. (TODO: use perfect derive.)
    impl<C: Context> PartialEq for InferredSchemaWithContext<C>
    where
        Schema<C>: PartialEq,
        C: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            self.schema == other.schema && self.context == other.context
        }
    }
}
