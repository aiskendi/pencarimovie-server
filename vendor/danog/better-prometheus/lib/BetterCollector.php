<?php

declare(strict_types=1);

namespace danog\BetterPrometheus;

use Prometheus\Collector;
use Prometheus\Storage\Adapter;

/**
 * A better prometheus collector.
 *
 * @api
 */
abstract class BetterCollector
{
    protected readonly string $metricName;

    /**
     * Constructor.
     */
    public function __construct(
        /** Storage adapter */
        public readonly Adapter $storageAdapter,
        /** Metric namespace */
        public readonly string $namespace,
        /** Metric name */
        public readonly string $name,
        /** Info about the metric */
        public readonly string $help,
        /** @var array<string, string> $labels Default labels, i.e. ['instance' => 'instance_1'] */
        public readonly array $labels = []
    ) {
        self::assertValidLabels($labels);
        $metricName = ($namespace !== '' ? $namespace . '_' : '') . $name;
        Collector::assertValidMetricName($metricName);
        $this->metricName = $metricName;
    }

    /** @param array<string, string> $labels */
    protected static function assertValidLabels(array $labels): void
    {
        foreach ($labels as $labelKey => $_) {
            Collector::assertValidLabel($labelKey);
        }
    }

    /**
     * Create a new instance of this collector, with these additional labels.
     *
     * @param array<string, string> $labels
     */
    abstract public function addLabels(array $labels): static;
}
