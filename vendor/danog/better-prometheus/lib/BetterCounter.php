<?php

declare(strict_types=1);

namespace danog\BetterPrometheus;

use Prometheus\Storage\Adapter;

/**
 * A better prometheus counter.
 *
 * @api
 */
final class BetterCounter extends BetterCollector
{
    private const TYPE = 'counter';

    public function addLabels(array $labels): static
    {
        return new self(
            $this->storageAdapter,
            $this->namespace,
            $this->name,
            $this->help,
            $this->labels + $labels
        );
    }

    /**
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function inc(array $labels = []): void
    {
        $this->incBy(1, $labels);
    }

    /**
     * @param int|float $count e.g. 2
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function incBy(int|float $count, array $labels = []): void
    {
        self::assertValidLabels($labels);
        $labels = $this->labels + $labels;
        $this->storageAdapter->updateCounter(
            [
                'name' => $this->metricName,
                'help' => $this->help,
                'type' => self::TYPE,
                'labelNames' => \array_keys($labels),
                'labelValues' => \array_values($labels),
                'value' => $count,
                'command' => \is_float($count) ? Adapter::COMMAND_INCREMENT_FLOAT : Adapter::COMMAND_INCREMENT_INTEGER,
            ]
        );
    }
}
