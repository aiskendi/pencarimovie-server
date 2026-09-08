<?php

declare(strict_types=1);

namespace danog\BetterPrometheus;

use Prometheus\Storage\Adapter;

/**
 * A better prometheus gauge.
 *
 * @api
 */
final class BetterGauge extends BetterCollector
{
    private const TYPE = 'gauge';

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
     * @param int|double $value e.g. 123
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function set(int|float $value, array $labels = []): void
    {
        self::assertValidLabels($labels);
        $labels = $this->labels + $labels;
        $this->storageAdapter->updateGauge(
            [
                'name' => $this->metricName,
                'help' => $this->help,
                'type' => self::TYPE,
                'labelNames' => \array_keys($labels),
                'labelValues' => \array_values($labels),
                'value' => $value,
                'command' => Adapter::COMMAND_SET,
            ]
        );
    }
    /**
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function incBy(int|float $value, array $labels = []): void
    {
        self::assertValidLabels($labels);
        $labels = $this->labels + $labels;
        $this->storageAdapter->updateGauge(
            [
                'name' => $this->metricName,
                'help' => $this->help,
                'type' => self::TYPE,
                'labelNames' => \array_keys($labels),
                'labelValues' => \array_values($labels),
                'value' => $value,
                'command' => \is_float($value) ? Adapter::COMMAND_INCREMENT_FLOAT : Adapter::COMMAND_INCREMENT_INTEGER,
            ]
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
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function dec(array $labels = []): void
    {
        $this->decBy(1, $labels);
    }

    /**
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function decBy(int|float $value, array $labels = []): void
    {
        $this->incBy(-$value, $labels);
    }
}
