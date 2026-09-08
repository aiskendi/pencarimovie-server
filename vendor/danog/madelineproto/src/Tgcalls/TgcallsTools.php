<?php declare(strict_types=1);

/**
 * This file is part of MadelineProto.
 * MadelineProto is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
 * MadelineProto is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU Affero General Public License for more details.
 * You should have received a copy of the GNU General Public License along with MadelineProto.
 * If not, see <http://www.gnu.org/licenses/>.
 *
 * @author    Daniil Gentili <daniil@daniil.it>
 * @copyright 2016-2025 Daniil Gentili <daniil@daniil.it>
 * @license   https://opensource.org/licenses/AGPL-3.0 AGPLv3
 * @link https://docs.madelineproto.xyz MadelineProto documentation
 */

// IMPORTANT NOTE: Please keep the above copyright notice intact if copying or rewriting this file in another language.

namespace danog\MadelineProto\Tgcalls;

use Amp\ByteStream\BufferedReader;
use Amp\ByteStream\ReadableBuffer;
use danog\MadelineProto\VoIP\SignalingProtocolVersion;

/** @internal */
final class TgcallsTools
{

    public static function deserializeRtc(
        SignalingProtocolVersion $tgcallsVersion,
        ?int $type,
        string $buffer
    ): array {
        if ($tgcallsVersion->isJson()) {
            return json_decode($buffer, true, flags: JSON_THROW_ON_ERROR);
        }
        $buffer = new BufferedReader(new ReadableBuffer($buffer));
        switch ($type) {
            case 1:
                $candidates = [];
                for ($x = \ord($buffer->readLength(1)); $x > 0; $x--) {
                    $candidates []= self::readString($buffer);
                }
                return [
                    '_' => 'candidatesList',
                    'ufrag' => self::readString($buffer),
                    'pwd' => self::readString($buffer),
                ];
            case 2:
                $formats = [];
                for ($x = \ord($buffer->readLength(1)); $x > 0; $x--) {
                    $name = self::readString($buffer);
                    $parameters = [];
                    for ($x = \ord($buffer->readLength(1)); $x > 0; $x--) {
                        $key = self::readString($buffer);
                        $value = self::readString($buffer);
                        $parameters[$key] = $value;
                    }
                    $formats[]= [
                        'name' => $name,
                        'parameters' => $parameters,
                    ];
                }
                return [
                    '_' => 'videoFormats',
                    'formats' => $formats,
                    'encoders' => \ord($buffer->readLength(1)),
                ];
            case 3:
                return ['_' => 'requestVideo'];
            case 4:
                $state = \ord($buffer->readLength(1));
                return ['_' => 'remoteMediaState', 'audio' => $state & 0x01, 'video' => ($state >> 1) & 0x03];
            case 5:
                return ['_' => 'audioData', 'data' => self::readBuffer($buffer)];
            case 6:
                return ['_' => 'videoData', 'data' => self::readBuffer($buffer)];
            case 7:
                return ['_' => 'unstructuredData', 'data' => self::readBuffer($buffer)];
            case 8:
                return ['_' => 'videoParameters', 'aspectRatio' => unpack('V', $buffer->readLength(4))[1]];
            case 9:
                return ['_' => 'remoteBatteryLevelIsLow', 'isLow' => (bool) \ord($buffer->readLength(1))];
            case 10:
                $lowCost = (bool) \ord($buffer->readLength(1));
                $isLowDataRequested = (bool) \ord($buffer->readLength(1));
                return ['_' => 'remoteNetworkStatus', 'lowCost' => $lowCost, 'isLowDataRequested' => $isLowDataRequested];
        }
        return ['_' => 'unknown', 'type' => $type];
    }

    public static function gunzip(string $data): string
    {
        if (\strlen($data) < 2) {
            return $data;
        }

        if (($data[0] == \chr(0x1f) && $data[1] == \chr(0x8b)) || ($data[0] == \chr(0x78) && $data[1] == \chr(0x9c))) {
            return gzdecode($data);
        }
        return $data;

    }

    private static function readString(BufferedReader $buffer): string
    {
        /** @psalm-suppress InvalidArgument */
        return $buffer->readLength(\ord($buffer->readLength(1)));
    }
    private static function readBuffer(BufferedReader $buffer): string
    {
        return $buffer->readLength(unpack('n', $buffer->readLength(2))[1]);
    }
}
