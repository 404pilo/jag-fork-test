# constraints

* [CONSTRAINT-01: The public release holds no private data](CONSTRAINT-01.md) - The public repository and package contain no private evaluation data, case content, keys, or internal names.
* [CONSTRAINT-02: jag protects the API key of the user](CONSTRAINT-02.md) - jag never shows the API key and refuses a key file that other users on the machine can read.
* [CONSTRAINT-03: A TypeSafe key is enough to start](CONSTRAINT-03.md) - A user needs only a TypeSafe API key to use jag; another service account is optional.
* [CONSTRAINT-04: jag tries again one time before it reports the service as down](CONSTRAINT-04.md) - Before jag reports the model service as down, it tries again one time and waits a fixed time that the user can change.
* [CONSTRAINT-05: The structured form only adds fields in one major version](CONSTRAINT-05.md) - In one major version, jag never removes or renames a field of structured output, and the output shows its form version.
* [CONSTRAINT-06: A typical search is one search on a medium code repository](CONSTRAINT-06.md) - The 5-second target applies to one search at the default cost limit, on a medium code repository, with a working service.
