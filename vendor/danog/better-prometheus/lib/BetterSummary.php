<?php

declare(strict_types=1);

namespace danog\BetterPrometheus;

use InvalidArgumentException;
use Prometheus\Storage\Adapter;
use Prometheus\Summary;

/**
 * A better prometheus summary.
 *
 * @api
 */
final class BetterSummary extends BetterCollector
{
    const RESERVED_LABELS = ['quantile'];
    const TYPE = 'summary';

    /**
     * @var non-empty-list<float>
     */
    private readonly array $quantiles;

    /**
     * @param array<string, string> $labels
     * @param ?non-empty-list<float> $quantiles
     */
    public function __construct(
        Adapter $adapter,
        string $namespace,
        string $name,
        string $help,
        array $labels = [],
        private readonly int $maxAgeSeconds = 600,
        ?array $quantiles = null
    ) {
        parent::__construct($adapter, $namespace, $name, $help, $labels);

        if (null === $quantiles) {
            /** @var non-empty-list<float> */
            $quantiles = Summary::getDefaultQuantiles();
        }

        /** @psalm-suppress TypeDoesNotContainType */
        if (0 === \count($quantiles)) {
            throw new InvalidArgumentException("Summary must have at least one quantile.");
        }

        for ($i = 0; $i < \count($quantiles) - 1; $i++) {
            if ($quantiles[$i] >= $quantiles[$i + 1]) {
                throw new InvalidArgumentException(
                    "Summary quantiles must be in increasing order: " .
                    $quantiles[$i] . " >= " . $quantiles[$i + 1]
                );
            }
        }

        foreach ($quantiles as $quantile) {
            if ($quantile <= 0 || $quantile >= 1) {
                throw new InvalidArgumentException("Quantile $quantile invalid: Expected number between 0 and 1.");
            }
        }

        if ($maxAgeSeconds <= 0) {
            throw new InvalidArgumentException("maxAgeSeconds $maxAgeSeconds invalid: Expected number greater than 0.");
        }

        $this->quantiles = $quantiles;
    }

    public function addLabels(array $labels): static
    {
        return new self(
            $this->storageAdapter,
            $this->namespace,
            $this->name,
            $this->help,
            $this->labels + $labels,
            $this->maxAgeSeconds,
            $this->quantiles
        );
    }

    /** @param array<string, string> $labels */
    protected static function assertValidLabels(array $labels): void
    {
        if (isset($labels['quantile'])) {
            throw new \InvalidArgumentException("Sumamry cannot have a label named 'quantile'.");
        }
        parent::assertValidLabels($labels);
    }

    /**
     * @param double|int $value e.g. 123
     * @param array<string, string> $labels e.g. ['status' => '404', 'opcode' => 'SOME_OP']
     */
    public function observe(float|int $value, array $labels = []): void
    {
        self::assertValidLabels($labels);
        $labels = $this->labels + $labels;

        $this->storageAdapter->updateSummary(
            [
                'value'         => $value,
                'name'          => $this->metricName,
                'help'          => $this->help,
                'type'          => self::TYPE,
                'labelNames'    => \array_keys($labels),
                'labelValues'   => \array_values($labels),
                'maxAgeSeconds' => $this->maxAgeSeconds,
                'quantiles'     => $this->quantiles,
            ]
        );
    }
}
