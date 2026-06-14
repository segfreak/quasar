# Linking rules
| Definition in existing | Incoming definition | Result |
|------------------------|---------------------|--------|
| –                      | any                 | use incoming |
| declaration only       | any                 | use incoming definition |
| strong definition      | strong definition   | error (`DuplicateDefinition`) |
| strong definition      | weak definition     | keep existing (warn) |
| weak definition        | strong definition   | use incoming (warn) |
| weak definition        | weak definition     | keep existing (warn) |