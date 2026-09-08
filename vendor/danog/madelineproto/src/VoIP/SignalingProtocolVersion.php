<?php declare(strict_types=1);

namespace danog\MadelineProto\VoIP;

/** @internal */
enum SignalingProtocolVersion
{
    case V1;
    case V2;
    case V3;

    case V1_JSON;
    case V2_JSON;

    public static function fromProtocol(array $protocol): self
    {
        $v = $protocol['library_versions'] ?? [];
        return self::fromLibraryVersion(end($v));
    }

    public static function fromLibraryVersion(string $version): self
    {
        return match ($version) {
            '7.0.0' => self::V1,
            '8.0.0' => self::V2,
            '9.0.0' => self::V2,

            '10.0.0' => self::V1_JSON,
            '11.0.0' => self::V2_JSON,

            '12.0.0' => self::V3,
            '13.0.0' => self::V3,

            default => throw new \InvalidArgumentException("Unknown VoIP signaling protocol version: $version"),
        };
    }

    public function isJson(): bool
    {
        return true;
        // return $this === self::V1_JSON || $this === self::V2_JSON;
    }
    public function supportsCompression(): bool
    {
        return $this === self::V3 || $this === self::V2_JSON;
    }
}
