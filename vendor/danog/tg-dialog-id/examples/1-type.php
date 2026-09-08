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
