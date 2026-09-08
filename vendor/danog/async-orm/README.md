# Async ORM

[![codecov](https://codecov.io/gh/danog/AsyncOrm/branch/master/graph/badge.svg)](https://codecov.io/gh/danog/AsyncOrm)
[![Psalm coverage](https://shepherd.dev/github/danog/AsyncOrm/coverage.svg)](https://shepherd.dev/github/danog/AsyncOrm)
[![Psalm level 1](https://shepherd.dev/github/danog/AsyncOrm/level.svg)](https://shepherd.dev/github/danog/AsyncOrm)
![License](https://img.shields.io/github/license/danog/AsyncOrm)

Async PHP ORM based on AMPHP v3 and fibers, created by Daniil Gentili (https://daniil.it) and Alexander Pankratov (https://github.com/xtrime-ru).  

Supports MySQL, Redis, Postgres.  

Features read and write-back caching, type-specific optimizations, and much more!  

This ORM library was initially created for [MadelineProto](https://docs.madelineproto.xyz), an async PHP client API for the telegram MTProto protocol.  

## Installation

```bash
composer require danog/async-orm
```

## Usage

There are two main ways to use the ORM: through automatic ORM properties, which automatically connects appropriately marked `DbArray` properties to the specified database, or by manually instantiating a `DbArray` with a `DbArrayBuilder`.

The `DbArray` obtained through one of the methods below is an abstract array object that automatically stores and fetches elements of the specified [type &raquo;](#value-types), from the specified database.  

`DbArray`s of type `ValueType::OBJECT` can contain objects extending `DbObject`.  

Classes extending `DbObject` have a special `save` method that can be used to persist object changes to the database, as can be seen in the [example](https://github.com/danog/AsyncOrm/blob/master/examples/2-manual.php).  


### Automatic ORM properties example

```php
<?php declare(strict_types=1);

use Amp\Mysql\MysqlConfig;
use Amp\Postgres\PostgresConfig;
use Amp\Redis\RedisConfig;
use danog\AsyncOrm\Annotations\OrmMappedArray;
use danog\AsyncOrm\DbArray;
use danog\AsyncOrm\DbAutoProperties;
use danog\AsyncOrm\DbObject;
use danog\AsyncOrm\KeyType;
use danog\AsyncOrm\Settings;
use danog\AsyncOrm\Settings\MysqlSettings;
use danog\AsyncOrm\Settings\PostgresSettings;
use danog\AsyncOrm\Settings\RedisSettings;
use danog\AsyncOrm\ValueType;

require __DIR__ . '/../vendor/autoload.php';

// Any of the following database backends can be used,
// remove the ones you don't need.
$settings = new MysqlSettings(
    new MysqlConfig(
        host: "/var/run/mysqld/mysqld.sock",
        user: 'user',
        password: 'password',
        database: 'database'
    ),
    cacheTtl: 100
);
$settings = new PostgresSettings(
    new PostgresConfig(
        host: "127.0.0.1",
        user: "user",
        password: "password",
        database: "database"
    ),
    cacheTtl: 100
);
$settings = new RedisSettings(
    RedisConfig::fromUri("redis://127.0.0.1"),
    cacheTtl: 100
);

/**
 * An object stored in a database.
 */
class MyObject extends DbObject
{
    public function __construct(
        private string $value
    ) {
    }
    public function setValue(string $value): void
    {
        $this->value = $value;
    }
    public function getValue(): string
    {
        return $this->value;
    }
}

/**
 * Main class of your application.
 */
final class Application
{
    use DbAutoProperties;

    /**
     * This field is automatically connected to the database using the specified Settings.
     *
     * @var DbArray<string, MyObject>
     */
    #[OrmMappedArray(KeyType::STRING, ValueType::OBJECT)]
    private DbArray $dbProperty1;

    /**
     * This field is automatically connected to the database using the specified Settings.
     *
     * @var DbArray<string, int>
     */
    #[OrmMappedArray(KeyType::STRING, ValueType::INT)]
    private DbArray $dbProperty2;

    public function __construct(
        Settings $settings,
        string $tablePrefix
    ) {
        $this->initDbProperties($settings, $tablePrefix);
    }

    public function businessLogic(): void
    {
        $this->dbProperty1['someOtherKey'] = new MyObject("initialValue");

        // Can store integers, strings, arrays or objects depending on the specified ValueType
        $this->dbProperty2['someKey'] = 123;
        var_dump($this->dbProperty2['someKey']);
    }

    public function businessLogic2(string $value): void
    {
        $obj = $this->dbProperty1['someOtherKey'];
        $obj->setValue($value);
        $obj->save();
    }

    public function businessLogic3(): string
    {
        return $this->dbProperty1['someOtherKey']->getValue();
    }

    public function shutdown(): void
    {
        // Flush all database caches, saving all changes.
        $this->saveDbProperties();
    }
}

$app = new Application($settings, 'tablePrefix');
$app->businessLogic();
$app->businessLogic2("newValue");
var_dump($app->businessLogic3());
$app->shutdown();
```

### Manual ORM properties example

See [here &raquo;](https://github.com/danog/AsyncOrm/blob/master/examples/2-manual.php)

### Settings

As specified in the examples above, there are multiple settings classes that can be used to connect to a specific database type:  

* [MysqlSettings: MySQL backend settings.](https://github.com/danog/AsyncOrm/blob/master/docs/docs/danog/AsyncOrm/Settings/MysqlSettings.md)
* [PostgresSettings: Postgres backend settings.](https://github.com/danog/AsyncOrm/blob/master/docs/docs/danog/AsyncOrm/Settings/PostgresSettings.md)
* [RedisSettings: Redis backend settings.](https://github.com/danog/AsyncOrm/blob/master/docs/docs/danog/AsyncOrm/Settings/RedisSettings.md)

All these classes have multiple fields, described in their respective documentation (click on each class name to view it).  

#### Caching

One of the most important settings is the `cacheTtl` field, which specifies the duration of the read and write cache.  

If non-zero, all array elements fetched from the database will be stored in an in-memory *read cache* for the specified number of seconds; multiple accesses to the same field will each postpone flushing of that field by `cacheTtl` seconds.  

All elements written to the array by the application will also be stored in an in-memory *write cache*, and flushed to the database every `cacheTtl` seconds.  

The cache is also flushed on shutdown by deferring an event loop callback, so make sure `EventLoop::run();` is being used to run the application, to make sure all data is flushed correctly (alternatively, `saveDbProperties` can be used to manually flush the cache on shutdown when using automatic properties, and `flushCache` in manual mode).  

If the array has an [object value type (ValueType::OBJECT)](#key-and-value-types), write caching is disabled.  

If `cacheTtl` is 0, read and write caching is disabled.  

A special setting class is used to create `DbArray`s backed by no database, which can also be useful in certain circumstances:  

* [MemorySettings: MemorySettings backend settings.](https://github.com/danog/AsyncOrm/blob/master/docs/docs/danog/AsyncOrm/Settings/MemorySettings.md)


### Key and value types

Each DbArray must have a specific key and value type.  

For optimal performance, the specified types must be as strict as possible, here's a list of allowed types:  

#### Key types

* `KeyType::STRING` - String keys only
* `KeyType::INT` - Integer keys only
* `KeyType::STRING_OR_INT` - String or integer keys (not recommended, for performance reasons please always specify either `STRING` or `STRING_OR_INT`).  

#### Value types

* `ValueType::STRING`: Direct storage of UTF-8 string values.
* `ValueType::INT`: Direct storage of integer values.
* `ValueType::BOOL`: Direct storage of boolean values.
* `ValueType::FLOAT`: Direct storage of floating point (double precision) values.
* `ValueType::SCALAR`: Values of any scalar type (including blobs and arrays, excluding objects), serialized as specified in the settings.
   Using SCALAR worsens performances, please use any of the other types if possible.
* `ValueType::OBJECT`: Objects extending DbObject, serialized as specified in the settings.

One of the most important value types is `ValueType::OBJECT`, it is used to store entire objects extending the `DbObject` class to the database.  

Objects extending `DbObject` have a special `save` method that can be used to persist object changes to the database, as can be seen in the [example](https://github.com/danog/AsyncOrm/blob/master/examples/2-manual.php).  

## API Documentation

Click [here &raquo;](https://github.com/danog/AsyncOrm/blob/master/docs/docs/index.md) to view the API documentation.
