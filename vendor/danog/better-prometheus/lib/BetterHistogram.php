<?php

declare(strict_types=1);

namespace danog\BetterPrometheus;

use InvalidArgumentException;
use Prometheus\Histogram;
use Prometheus\Storage\Adapter;

/**
 * A better prometheus histogram.
 *
 * @api
 */
final class BetterHistogram extends BetterCollector
{
    const TYPE = 'histogram';

    /**
     * @var non-empty-list<float>
     */
    private readonly array $buckets;

    /**
     * @param array<string, string> $labels
     * @param non-empty-list<float>|null $buckets
     */
    public function __construct(
        Adapter $adapter,
        string $namespace,
        string $name,
        string $help,
        array $labels = [],
        ?array $buckets = null
    ) {
        parent::__construct($adapter, $namespace, $name, $help, $labels);

        if (null === $buckets) {
            /** @var non-empty-list<float> */
            $buckets = Histogram::getDefaultBuckets();
        }

        /** @psalm-suppress TypeDoesNotContainType */
        if (0 === \count($buckets)) {
            throw new InvalidArgumentException("Histogram must have at least one bucket.");
        }

        for ($i = 0; $i < \count($buckets) - 1; $i++) {
            if ($buckets[$i] >= $buckets[$i + 1]) {
                throw new InvalidArgumentException(
                    "Histogram buckets must be in increasing order: " .
                    $buckets[$i] . " >= " . $buckets[$i + 1]
                );
            }
        }
        $this->buckets = $buckets;
    }

    /** @param array<string, string> $labels */
    protected static function assertValidLabels(array $labels): void
    {
        if (isset($labels['le'])) {
            throw new \InvalidArgumentException("Histogram cannot have a label named 'le'.");
        }
        parent::assertValidLabels($labels);
    }

    public function addLabels(array $labels): static
    {
        return new self(
            $this->storageAdapter,
            $this->namespace,
            $this->name,
            $this->help,
            $this->labels + $labels,
            $this->buckets
        );
    }

    /**
     * @param double|int $value e.g. 123
     * @param array<string, string> $labels e.g. ['status' => '201', 'opcode' => 'SOME_OP']
     */
    public function observe(float|int $value, array $labels = []): void
    {
        self::assertValidLabels($labels);
        $labels = $this->labels + $labels;

        $this->storageAdapter->updateHistogram(
            [
                'value'       => $value,
                'name'        => $this->metricName,
                'help'        => $this->help,
                'type'        => self::TYPE,
                'labelNames'  => \array_keys($labels),
                'labelValues' => \array_values($labels),
                'buckets'     => $this->buckets,
            ]
        );
    }
}
