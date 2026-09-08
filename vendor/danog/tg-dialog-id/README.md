# tg-dialog-id

[![codecov](https://codecov.io/gh/danog/tg-dialog-id/branch/master/graph/badge.svg)](https://codecov.io/gh/danog/tg-dialog-id)
[![Psalm coverage](https://shepherd.dev/github/danog/tg-dialog-id/coverage.svg)](https://shepherd.dev/github/danog/tg-dialog-id)
[![Psalm level 1](https://shepherd.dev/github/danog/tg-dialog-id/level.svg)](https://shepherd.dev/github/danog/tg-dialog-id)
[![Mutation testing badge](https://img.shields.io/endpoint?style=flat&url=https%3A%2F%2Fbadge-api.stryker-mutator.io%2Fgithub.com%2Fdanog%2Ftg-dialog-id%2Fmaster)](https://dashboard.stryker-mutator.io/reports/github.com/danog/tg-dialog-id/master)
![License](https://img.shields.io/github/license/danog/tg-dialog-id)

A library to work with Telegram bot API dialog IDs.  

Created by Daniil Gentili (https://daniil.it).  

This library was initially created for [MadelineProto](https://docs.madelineproto.xyz), an async PHP client API for the telegram MTProto protocol.  

## Installation

```bash
composer require danog/tg-dialog-id
```

## Usage

```php
<?php declare(strict_types=1);

use danog\DialogId\DialogId;

require 'vendor/autoload.php';

function expect(bool $expect): void
{
    if (!$expect) {
        throw new AssertionError("Not verified!");
    }
}

$link = "https://t.me/c/1234567890/8892";
if (preg_match("|t.me/c/(\d+)/(\d+)|", $link, $matches)) {
    // Returns -1001234567890
    echo DialogId::fromSupergroupOrChannelId((int) $matches[1]).PHP_EOL;
}

// Converts an MTProto supergroup/channel ID => bot API dialog ID
expect(DialogId::fromSupergroupOrChannelId(1234567890) === -1001234567890);

// Converts an MTProto chat ID => bot API dialog ID
expect(DialogId::fromChatId(123456789) === -123456789);

// Converts an MTProto user ID => bot API dialog ID
expect(DialogId::fromUserId(123456789) === 123456789);

// Converts an MTProto secret chat ID => bot API dialog ID
expect(DialogId::fromSecretChatId(123456789) === -1999876543211);

// Converts a bot API dialog ID => MTProto supergroup/channel ID
expect(DialogId::toSupergroupOrChannelId(-1001234567890) === 1234567890);

// Converts a bot API dialog ID => MTProto chat ID
expect(DialogId::toChatId(-123456789) === 123456789);

// Converts a bot API dialog ID => MTProto user ID
expect(DialogId::toUserId(123456789) === 123456789);

// Converts a bot API dialog ID => MTProto secret chat ID
expect(DialogId::toSecretChatId(-1999876543211) === 123456789);

expect(DialogId::getType(101374607) === DialogId::USER);
expect(DialogId::getType(-123456789) === DialogId::CHAT);
expect(DialogId::getType(-1001234567890) === DialogId::CHANNEL_OR_SUPERGROUP);
expect(DialogId::getType(-1999898625393) === DialogId::SECRET_CHAT);

expect(DialogId::isUser(1099511627775) === true);
expect(DialogId::isChat(-999_999_999_999) === true);
expect(DialogId::isSupergroupOrChannel(-1997852516352) === true);
expect(DialogId::isSecretChat(-2002147483648) === true);

expect(DialogId::isUser(101374607) === true);
expect(DialogId::isChat(-101374607) === true);
expect(DialogId::isSupergroupOrChannel(-1001234567890) === true);
expect(DialogId::isSecretChat(-1999898625393) === true);

// Converts a bot API dialog ID => MTProto ID automatically depending on type
expect(DialogId::toMTProtoId(-1001234567890) === 1234567890);
expect(DialogId::toMTProtoId(-123456789) === 123456789);
expect(DialogId::toMTProtoId(123456789) === 123456789);
expect(DialogId::toMTProtoId(-1999876543211) === 123456789);

echo "OK!".PHP_EOL;
```

## API Documentation

Click [here &raquo;](https://github.com/danog/tg-dialog-id/blob/master/docs/index.md) to view the API documentation.
