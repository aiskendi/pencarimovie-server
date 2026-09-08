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
use danog\MadelineProto\Exception;
use danog\MadelineProto\MTProto;
use danog\MadelineProto\MTProtoTools\Crypt;
use danog\MadelineProto\VoIP\SignalingProtocolVersion;
use Webrtc\DataChannel\RTCDataChannel;
use Webrtc\DataChannel\RTCDataChannelParameters;
use Webrtc\ICE\Enum\IceGatheringState;
use Webrtc\ICE\RTCIceCandidate;
use Webrtc\Webrtc\RTCPeerConnection;

use function React\Async\await;

/** @internal */
final class Controller
{

    private const SIGNALING_MIN_SIZE = 21;
    private const SIGNALING_MAX_SIZE = 128 * 1024 * 1024;

    private const SINGLE_MESSAGE_PACKET_BIT = 1 << 31;
    private const MESSAGE_REQUIRES_ACK_SEQ_BIT = 1 << 30;

    private const MAX_ALLOWED_COUNTER = ~self::SINGLE_MESSAGE_PACKET_BIT
        & ~self::MESSAGE_REQUIRES_ACK_SEQ_BIT;

    public const ACK_ID = 255;
    public const EMPTY_ID = 254;
    public const CUSTOM_ID = 127;

    private RTCPeerConnection $peerConnection;
    private RTCDataChannel $dataChannel;

    private int $remoteSeq = 0;
    private int $localSeq = 0;

    public function __construct(
        private readonly string $authKey,
        private readonly bool $outgoing,
        private readonly SignalingProtocolVersion $tgcallsVersion,
        private readonly MTProto $API,
        array $connections
    ) {
        $iceServers = [];
        foreach ($connections as $connection) {
            if ($connection['_'] !== 'phoneConnectionWebrtc') {
                continue;
            }
            foreach ([
                $connection['ip'],
                '['.$connection['ipv6'].']',
            ] as $ip) {
                if ($connection['turn']) {
                    $url = 'turn:'.$ip.':'.$connection['port'];
                } elseif ($connection['stun']) {
                    $url = 'stun:'.$ip.':'.$connection['port'];
                } else {
                    continue;
                }
                $iceServers[] = [
                    'urls' => $url,
                    'username' => $connection['username'],
                    'credential' => $connection['password'],
                    'credentialType' => 'password',
                ];
            }
        }
        $this->peerConnection = new RTCPeerConnection([
            'iceServers' => $iceServers,
        ]);
        if ($this->outgoing) {
            $this->dataChannel = $this->peerConnection->createDataChannel(new RTCDataChannelParameters(
                "data"
            ));
        }

        $offer = await($this->peerConnection->createOffer());
        await($this->peerConnection->setLocalDescription($offer));

        $this->sendSignalling([
            'type' => $offer->getType(),
            'sdp' => $offer->getSdp(),
        ]);

        $this->peerConnection->on('icegatheringstatechange', function (): void {
            if ($this->peerConnection->getIceGatheringState() !== IceGatheringState::complete) {
                return;
            }

            foreach ($this->peerConnection->getTransceivers() as $transceiver) {
                $iceGatherer = $transceiver->getSender()->getTransport()->getIceTransport()->getIceGatherer();
                $candidates = [];
                foreach ($iceGatherer->getLocalCandidates() as $candidate) {
                    $candidate->setSdpMid($transceiver->getMid());
                    $candidates[] = [
                        'sdpString' => $candidate->toSDP(),
                    ];
                }

                $this->sendSignalling([
                    '@type' => 'Candidates',
                    'candidates' => $candidates,
                ]);
            }
        });
    }

    public function sendSignalling(array $message): void
    {
        $seq = $this->localSeq++;

        $serialized = TgcallsTools::serializeRtc($this->tgcallsVersion, $message);
        if ($this->tgcallsVersion->supportsCompression()) {
            $serialized = TgcallsTools::gzip($serialized);
        }
        $serialized = pack('N', $seq).$serialized;

        $serialized = $this->encryptPayload($serialized, false);
    }

    private function encryptPayload(string $serialized, bool $signaling): void
    {
        $x = Crypt::voipX(!$this->outgoing, $signaling);
        $message_key_full = hash('sha256', substr($this->authKey, 88 + $x, 32).$serialized, true);
        $message_key = substr($message_key_full, 8, 16);
        [$aes_key, $aes_iv, $x] = Crypt::voipKdf($message_key, $this->authKey, $x);
        $packet = Crypt::ctrEncrypt($serialized, $aes_key, $aes_iv);

        $data = $message_key.$packet;

        // send $data to peer
    }
    public function onSignaling(string $data): void
    {
        if ($this->tgcallsVersion === null) {
            throw new Exception('Protocol version is not set!');
        }
        if (\strlen($data) < self::SIGNALING_MIN_SIZE || \strlen($data) > self::SIGNALING_MAX_SIZE) {
            throw new Exception('Invalid signaling size!');
        }
        $message_key = substr($data, 0, 16);
        $data = substr($data, 16);

        $x = Crypt::voipX($this->outgoing, true);
        [$aes_key, $aes_iv, $x] = Crypt::voipKdf($message_key, $this->authKey, $x);
        $packet = Crypt::ctrEncrypt($data, $aes_key, $aes_iv);

        if ($message_key != substr(hash('sha256', substr($this->authKey, 88 + $x, 32).$packet, true), 8, 16)) {
            throw new Exception('msg_key mismatch!');
        }
        if (\strlen($packet) < self::SIGNALING_MIN_SIZE || \strlen($packet) > self::SIGNALING_MAX_SIZE) {
            throw new Exception('Invalid signaling size!');
        }

        if ($this->tgcallsVersion->supportsCompression()) {
            $packet = TgcallsTools::gunzip($packet);

            $seq = unpack('N', substr($packet, 0, 4))[1];

            $this->onSignalingMessage(TgcallsTools::deserializeRtc(
                $this->tgcallsVersion,
                null,
                substr($packet, 4)
            ));
            return;
        }

        $packet = new BufferedReader(new ReadableBuffer($packet));

        $first = true;
        while ($packet->isReadable()) {
            $seq = unpack('N', $packet->readLength(4))[1];
            $messageRequiresAck = (bool) ($seq & self::MESSAGE_REQUIRES_ACK_SEQ_BIT);
            $singlePacketFlag = (bool) ($seq & self::SINGLE_MESSAGE_PACKET_BIT);

            if (!$first && $singlePacketFlag) {
                throw new Exception('Single packet flag can only be set on first message!');
            }

            $type = \ord($packet->readLength(1));
            if ($type === self::EMPTY_ID) {
                if (!$first) {
                    throw new Exception('Empty packet can only be first message!');
                }
            } elseif ($type === self::ACK_ID) {
                // todo ack $seq (contains my seq to be acked)
            } else {
                $length = unpack('N', $packet->readLength(4))[1];
                if ($length > 1024 * 1024) {
                    throw new Exception('Invalid signaling message length!');
                }
                $str = $packet->readLength($length);
                if (\strlen($str) !== $length) {
                    throw new Exception('Signaling message is shorter than expected!');
                }

                $this->onSignalingMessage(TgcallsTools::deserializeRtc($this->tgcallsVersion, $type, $str));
            }
            $first = false;
        }

    }
    private function onSignalingMessage(array $message): void
    {
        if ($this->tgcallsVersion->isJson()) {
            $this->onSignalingMessageJson($message);
            return;
        }
    }

    private function onSignalingMessageJson(array $message): void
    {
        $type = $message['@type'];
        if ($type === 'Candidates') {
            foreach ($message['candidates'] as ['sdpString' => $sdp]) {
                $candidate = RTCIceCandidate::parseSDP($sdp);
                $candidate->setSdpMid(0);
                $this->peerConnection->addIceCandidate($candidate);
            }
            return;
        }
        var_dump($message);
        readline();
    }

}
