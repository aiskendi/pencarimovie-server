<?php declare(strict_types=1);

namespace danog\TestDialogId;

use AssertionError;
use danog\DialogId\DialogId;
use PHPUnit\Framework\TestCase;

final class DialogIdTest extends TestCase
{
    public function testAll(): void
    {
        $this->assertSame(DialogId::USER, DialogId::getType(101374607));
        $this->assertSame(DialogId::CHAT, DialogId::getType(-101374607));
        $this->assertSame(DialogId::CHANNEL_OR_SUPERGROUP, DialogId::getType(-1001234567890));
        $this->assertSame(DialogId::SECRET_CHAT, DialogId::getType(-1999898625393));
        $this->assertSame(DialogId::MONOFORUM, DialogId::getType(-2002147483649));
        $this->assertSame(DialogId::MONOFORUM, DialogId::getType(-4000000000000));

        $this->assertTrue(DialogId::isUser(1099511627775));
        $this->assertTrue(DialogId::isChat(-999_999_999_999));
        $this->assertTrue(DialogId::isSupergroupOrChannel(-1997852516352));
        $this->assertTrue(DialogId::isSecretChat(-2002147483648));
        $this->assertTrue(DialogId::isMonoforum(-2002147483649));

        $this->assertTrue(DialogId::isUser(101374607));
        $this->assertTrue(DialogId::isChat(-101374607));
        $this->assertTrue(DialogId::isSupergroupOrChannel(-1001234567890));
        $this->assertTrue(DialogId::isSupergroupOrChannelOrMonoforum(-1001234567890));
        $this->assertTrue(DialogId::isSecretChat(-1999898625393));
        $this->assertTrue(DialogId::isMonoforum(-4000000000000));
        $this->assertTrue(DialogId::isSupergroupOrChannelOrMonoforum(-4000000000000));

        $this->assertFalse(DialogId::isSupergroupOrChannelOrMonoforum(1));

        $this->assertSame(101374607, DialogId::toUserId(101374607));
        $this->assertSame(101374607, DialogId::toChatId(-101374607));
        $this->assertSame(1234567890, DialogId::toSupergroupOrChannelId(-1001234567890));
        $this->assertSame(101374607, DialogId::toSecretChatId(-1999898625393));
        $this->assertSame(1002147483649, DialogId::toMonoforumId(-2002147483649));

        $this->assertSame(101374607, DialogId::toMTProtoId(101374607));
        $this->assertSame(101374607, DialogId::toMTProtoId(-101374607));
        $this->assertSame(1234567890, DialogId::toMTProtoId(-1001234567890));
        $this->assertSame(101374607, DialogId::toMTProtoId(-1999898625393));
        $this->assertSame(1002147483649, DialogId::toMTProtoId(-2002147483649));

        $this->assertSame(101374607, DialogId::fromUserId(101374607));
        $this->assertSame(-101374607, DialogId::fromChatId(101374607));
        $this->assertSame(-1001234567890, DialogId::fromSupergroupOrChannelId(1234567890));
        $this->assertSame(-1999898625393, DialogId::fromSecretChatId(101374607));
        $this->assertSame(-2002147483649, DialogId::fromMonoforumId(1002147483649));
    }

    public function testException1(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Invalid ID -4000000000001 provided!");
        DialogId::getType(-4000000000001);
    }
    public function testException2(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Invalid ID 0 provided!");
        $this->assertTrue(DialogId::getType(0));
    }
    public function testException3(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a chat ID, but produced the following type: USER");
        DialogId::fromChatId(-100);
    }
    public function testException4(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a user ID, but produced the following type: CHAT");
        DialogId::fromUserId(-100);
    }
    public function testException5(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a supergroup/channel ID, but produced the following type: CHAT");
        DialogId::fromSupergroupOrChannelId(-100);
    }
    public function testException6(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a secret chat ID, but produced the following type: CHAT");
        DialogId::fromSecretChatId((1 << 40) - 1);
    }
    public function testException7(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a monoforum ID, but produced the following type: CHAT");
        DialogId::fromMonoforumId(-100);
    }

    public function testException3_rev(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a chat ID, got the following type: USER");
        DialogId::toChatId(100);
    }
    public function testException4_rev(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a user ID, got the following type: CHAT");
        DialogId::toUserId(-100);
    }
    public function testException5_rev(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a supergroup/channel ID, got the following type: CHAT");
        DialogId::toSupergroupOrChannelId(-100);
    }
    public function testException6_rev(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a secret chat ID, got the following type: USER");
        DialogId::toSecretChatId((1 << 40) - 1);
    }
    public function testException7_rev(): void
    {
        $this->expectException(AssertionError::class);
        $this->expectExceptionMessage("Expected a monoforum ID, got the following type: USER");
        DialogId::toMonoforumId((1 << 40) - 1);
    }
}
