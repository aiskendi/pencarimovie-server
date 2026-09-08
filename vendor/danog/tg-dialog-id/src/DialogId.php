<?php declare(strict_types=1);

namespace danog\DialogId;

use AssertionError;

/**
 * Represents the type of a bot API dialog ID.
 *
 * @psalm-immutable
 *
 * @api
 */
enum DialogId
{
    private const ZERO_CHANNEL_ID = -1000000000000;
    private const ZERO_SECRET_CHAT_ID = -2000000000000;

    private const MAX_USER_ID = (1 << 40) - 1;

    private const MIN_CHAT_ID = -999_999_999_999;

    private const MIN_CHANNEL_ID = self::ZERO_CHANNEL_ID - (1000000000000 - (1 << 31));

    private const MIN_SECRET_CHAT_ID = self::ZERO_SECRET_CHAT_ID - 2147483648;

    private const MIN_MONOFORUM_CHANNEL_ID = self::ZERO_CHANNEL_ID - 3000000000000;

    /**
     * Dialog type: user.
     */
    case USER;
    /**
     * Dialog type: chat.
     */
    case CHAT;
    /**
     * Dialog type: supergroup or channel, see https://core.telegram.org/api/channel for more info.
     */
    case CHANNEL_OR_SUPERGROUP;
    /**
     * Dialog type: secret chat.
     */
    case SECRET_CHAT;
    /**
     * Dialog type: monoforum.
     */
    case MONOFORUM;

    /**
     * Get the type of a dialog using just its bot API dialog ID.
     *
     * @psalm-pure
     *
     * @param integer $id Bot API ID.
     */
    public static function getType(int $id): self
    {
        if ($id < 0) {
            if (self::MIN_CHAT_ID <= $id) {
                return DialogId::CHAT;
            }
            if (self::MIN_CHANNEL_ID <= $id && $id !== self::ZERO_CHANNEL_ID) {
                return DialogId::CHANNEL_OR_SUPERGROUP;
            }
            if (self::MIN_SECRET_CHAT_ID <= $id && $id !== self::ZERO_SECRET_CHAT_ID) {
                return DialogId::SECRET_CHAT;
            }
            if (self::MIN_MONOFORUM_CHANNEL_ID <= $id) {
                return DialogId::MONOFORUM;
            }
        } elseif ($id > 0) {
            return DialogId::USER;
        }
        throw new AssertionError("Invalid ID $id provided!");
    }

    /**
     * Checks whether the provided bot API ID is a supergroup or channel ID.
     *
     * @psalm-pure
     */
    public static function isSupergroupOrChannel(int $id): bool
    {
        return self::getType($id) === self::CHANNEL_OR_SUPERGROUP;
    }

    /**
     * Checks whether the provided bot API ID is a supergroup, channel or monoforum.
     *
     * @psalm-pure
     */
    public static function isSupergroupOrChannelOrMonoforum(int $id): bool
    {
        $t = self::getType($id);
        return $t === self::CHANNEL_OR_SUPERGROUP || $t === self::MONOFORUM;
    }

    /**
     * Checks whether the provided bot API ID is a monoforum.
     *
     * @psalm-pure
     */
    public static function isMonoforum(int $id): bool
    {
        return self::getType($id) === self::MONOFORUM;
    }

    /**
     * Checks whether the provided bot API ID is a chat ID.
     *
     * @psalm-pure
     */
    public static function isChat(int $id): bool
    {
        return self::getType($id) === self::CHAT;
    }
    /**
     * Checks whether the provided bot API ID is a user ID.
     *
     * @psalm-pure
     */
    public static function isUser(int $id): bool
    {
        return self::getType($id) === self::USER;
    }
    /**
     * Checks whether the provided bot API ID is a secret chat ID.
     *
     * @psalm-pure
     */
    public static function isSecretChat(int $id): bool
    {
        return self::getType($id) === self::SECRET_CHAT;
    }

    /**
     * Convert MTProto secret chat ID to bot API secret chat ID.
     *
     * @psalm-pure
     *
     * @param int $id MTProto secret chat ID
     *
     * @return int Bot API secret chat ID
     */
    public static function fromSecretChatId(int $id): int
    {
        $id += self::ZERO_SECRET_CHAT_ID;
        $type = self::getType($id);
        if ($type !== self::SECRET_CHAT) {
            throw new AssertionError("Expected a secret chat ID, but produced the following type: ".$type->name);
        }
        return $id;
    }
    /**
     * Convert bot API secret chat ID to MTProto secret chat ID.
     *
     * @psalm-pure
     *
     * @param int $id Bot API secret chat ID
     *
     * @return int MTProto secret chat ID
     */
    public static function toSecretChatId(int $id): int
    {
        $type = self::getType($id);
        if ($type !== self::SECRET_CHAT) {
            throw new AssertionError("Expected a secret chat ID, got the following type: ".$type->name);
        }
        return $id - self::ZERO_SECRET_CHAT_ID;
    }

    /**
     * Convert MTProto channel ID to bot API channel ID.
     *
     * @psalm-pure
     *
     * @param int $id MTProto channel ID
     */
    public static function fromSupergroupOrChannelId(int $id): int
    {
        $id = self::ZERO_CHANNEL_ID - $id;
        $type = self::getType($id);
        if ($type !== self::CHANNEL_OR_SUPERGROUP) {
            throw new AssertionError("Expected a supergroup/channel ID, but produced the following type: ".$type->name);
        }
        return $id;
    }
    /**
     * Convert bot API channel ID to MTProto channel ID.
     *
     * @psalm-pure
     *
     * @param int $id Bot API channel ID
     */
    public static function toSupergroupOrChannelId(int $id): int
    {
        $type = self::getType($id);
        if ($type !== self::CHANNEL_OR_SUPERGROUP) {
            throw new AssertionError("Expected a supergroup/channel ID, got the following type: ".$type->name);
        }
        return (-$id) + self::ZERO_CHANNEL_ID;
    }

    /**
     * Convert MTProto chat ID to bot API chat ID.
     *
     * @psalm-pure
     *
     * @param int $id MTProto chat ID
     */
    public static function fromChatId(int $id): int
    {
        $id = -$id;
        $type = self::getType($id);
        if ($type !== self::CHAT) {
            throw new AssertionError("Expected a chat ID, but produced the following type: ".$type->name);
        }
        return $id;
    }
    /**
     * Convert bot API chat ID to MTProto chat ID.
     *
     * @psalm-pure
     *
     * @param int $id Bot API chat ID
     */
    public static function toChatId(int $id): int
    {
        $type = self::getType($id);
        if ($type !== self::CHAT) {
            throw new AssertionError("Expected a chat ID, got the following type: ".$type->name);
        }
        return -$id;
    }

    /**
     * Convert MTProto monoforum ID to bot API monoforum ID.
     *
     * @psalm-pure
     *
     * @param int $id MTProto monoforum ID
     */
    public static function fromMonoforumId(int $id): int
    {
        $id = self::ZERO_CHANNEL_ID - $id;
        $type = self::getType($id);
        if ($type !== self::MONOFORUM) {
            throw new AssertionError("Expected a monoforum ID, but produced the following type: ".$type->name);
        }
        return $id;
    }

    /**
     * Convert bot API monoforum ID to MTProto monoforum ID.
     *
     * @psalm-pure
     *
     * @param int $id Bot API monoforum ID
     */
    public static function toMonoforumId(int $id): int
    {
        $type = self::getType($id);
        if ($type !== self::MONOFORUM) {
            throw new AssertionError("Expected a monoforum ID, got the following type: ".$type->name);
        }
        return (-$id) + self::ZERO_CHANNEL_ID;
    }

    /**
     * Convert MTProto user ID to bot API user ID.
     *
     * @psalm-pure
     *
     * @param int $id MTProto user ID
     */
    public static function fromUserId(int $id): int
    {
        $type = self::getType($id);
        if ($type !== self::USER) {
            throw new AssertionError("Expected a user ID, but produced the following type: ".$type->name);
        }
        return $id;
    }
    /**
     * Convert bot API user ID to MTProto user ID.
     *
     * @psalm-pure
     *
     * @param int $id Bot API user ID
     */
    public static function toUserId(int $id): int
    {
        $type = self::getType($id);
        if ($type !== self::USER) {
            throw new AssertionError("Expected a user ID, got the following type: ".$type->name);
        }
        return $id;
    }

    /**
     * Convert bot API ID to MTProto ID (automatically detecting the correct type).
     *
     * @psalm-pure
     *
     * @param int $id Bot API dialog ID
     */
    public static function toMTProtoId(int $id): int
    {
        return match (self::getType($id)) {
            self::USER => self::toUserId($id),
            self::MONOFORUM => self::toMonoforumId($id),
            self::CHAT => self::toChatId($id),
            self::CHANNEL_OR_SUPERGROUP => self::toSupergroupOrChannelId($id),
            self::SECRET_CHAT => self::toSecretChatId($id)
        };
    }
}
