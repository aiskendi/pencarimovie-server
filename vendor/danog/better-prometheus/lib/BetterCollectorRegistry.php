<?php

declare(strict_types=1);

namespace danog\BetterPrometheus;

use Prometheus\Exception\MetricNotFoundException;
use Prometheus\Exception\MetricsRegistrationException;
use Prometheus\MetricFamilySamples;
use Prometheus\Storage\Adapter;

/**
 * A better collector registry.
 *
 * @api
 */
final class BetterCollectorRegistry
{
    /**
     * @var array<string, BetterGauge>
     */
    private array $gauges = [];

    /**
     * @var array<string, BetterCounter>
     */
    private array $counters = [];

    /**
     * @var array<string, BetterHistogram>
     */
    private array $histograms = [];

    /**
     * @var array<string, BetterSummary>
     */
    private array $summaries = [];

    /**
     * @psalm-suppress UnusedProperty
     */
    private ?BetterGauge $defaultGauge = null;

    /**
     * CollectorRegistry constructor.
     *
     */
    public function __construct(
        public readonly Adapter $storageAdapter,
        bool $registerDefaultMetrics = true
    ) {
        if ($registerDefaultMetrics) {
            $this->defaultGauge = $this->getOrRegisterGauge(
                "",
                "php_info",
                "Information about the PHP environment.",
                ["version" => PHP_VERSION]
            );
            $this->defaultGauge->set(1);
        }
    }

    /**
     * Removes all previously stored metrics from underlying storage adapter.
     */
    public function wipeStorage(): void
    {
        $this->storageAdapter->wipeStorage();
    }

    /**
     * @psalm-suppress TooManyArguments
     *
     * @return list<MetricFamilySamples>
     */
    public function getMetricFamilySamples(bool $sortMetrics = true): array
    {
        /** @var list<MetricFamilySamples> */
        return $this->storageAdapter->collect($sortMetrics);
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. duration_seconds
     * @param string $help e.g. The duration something took in seconds.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     *
     * @throws MetricsRegistrationException
     */
    public function registerGauge(string $namespace, string $name, string $help, array $labels = []): BetterGauge
    {
        $metricIdentifier = "$namespace:$name";
        if (isset($this->gauges[$metricIdentifier])) {
            throw new MetricsRegistrationException("Metric already registered");
        }
        $this->gauges[$metricIdentifier] = new BetterGauge(
            $this->storageAdapter,
            $namespace,
            $name,
            $help,
            $labels
        );
        return $this->gauges[$metricIdentifier];
    }

    /**
     * @throws MetricNotFoundException
     */
    public function getGauge(string $namespace, string $name): BetterGauge
    {
        $metricIdentifier = "$namespace:$name";
        if (!isset($this->gauges[$metricIdentifier])) {
            throw new MetricNotFoundException("Metric not found:" . $metricIdentifier);
        }
        return $this->gauges[$metricIdentifier];
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. duration_seconds
     * @param string $help e.g. The duration something took in seconds.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     *
     * @throws MetricsRegistrationException
     */
    public function getOrRegisterGauge(string $namespace, string $name, string $help, array $labels = []): BetterGauge
    {
        try {
            $gauge = $this->getGauge($namespace, $name);
        } catch (MetricNotFoundException $e) {
            $gauge = $this->registerGauge($namespace, $name, $help, $labels);
        }
        return $gauge;
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. requests
     * @param string $help e.g. The number of requests made.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     *
     * @throws MetricsRegistrationException
     */
    public function registerCounter(string $namespace, string $name, string $help, array $labels = []): BetterCounter
    {
        $metricIdentifier = "$namespace:$name";
        if (isset($this->counters[$metricIdentifier])) {
            throw new MetricsRegistrationException("Metric already registered");
        }
        $this->counters[$metricIdentifier] = new BetterCounter(
            $this->storageAdapter,
            $namespace,
            $name,
            $help,
            $labels
        );
        return $this->counters["$namespace:$name"];
    }

    /**
     *
     * @throws MetricNotFoundException
     */
    public function getCounter(string $namespace, string $name): BetterCounter
    {
        $metricIdentifier = "$namespace:$name";
        if (!isset($this->counters[$metricIdentifier])) {
            throw new MetricNotFoundException("Metric not found:" . $metricIdentifier);
        }
        return $this->counters["$namespace:$name"];
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. requests
     * @param string $help e.g. The number of requests made.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     *
     * @throws MetricsRegistrationException
     */
    public function getOrRegisterCounter(string $namespace, string $name, string $help, array $labels = []): BetterCounter
    {
        try {
            $counter = $this->getCounter($namespace, $name);
        } catch (MetricNotFoundException $e) {
            $counter = $this->registerCounter($namespace, $name, $help, $labels);
        }
        return $counter;
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. duration_seconds
     * @param string $help e.g. A histogram of the duration in seconds.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     * @param non-empty-list<float>|null $buckets e.g. [100, 200, 300]
     *
     * @throws MetricsRegistrationException
     */
    public function registerHistogram(
        string $namespace,
        string $name,
        string $help,
        array $labels = [],
        ?array $buckets = null
    ): BetterHistogram {
        $metricIdentifier = "$namespace:$name";
        if (isset($this->histograms[$metricIdentifier])) {
            throw new MetricsRegistrationException("Metric already registered");
        }
        $this->histograms[$metricIdentifier] = new BetterHistogram(
            $this->storageAdapter,
            $namespace,
            $name,
            $help,
            $labels,
            $buckets
        );
        return $this->histograms[$metricIdentifier];
    }

    /**
     *
     * @throws MetricNotFoundException
     */
    public function getHistogram(string $namespace, string $name): BetterHistogram
    {
        $metricIdentifier = "$namespace:$name";
        if (!isset($this->histograms[$metricIdentifier])) {
            throw new MetricNotFoundException("Metric not found:" . $metricIdentifier);
        }
        return $this->histograms["$namespace:$name"];
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. duration_seconds
     * @param string $help e.g. A histogram of the duration in seconds.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     * @param non-empty-list<float>|null $buckets e.g. [100, 200, 300]
     *
     * @throws MetricsRegistrationException
     */
    public function getOrRegisterHistogram(
        string $namespace,
        string $name,
        string $help,
        array $labels = [],
        ?array $buckets = null
    ): BetterHistogram {
        try {
            $histogram = $this->getHistogram($namespace, $name);
        } catch (MetricNotFoundException $e) {
            $histogram = $this->registerHistogram($namespace, $name, $help, $labels, $buckets);
        }
        return $histogram;
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. duration_seconds
     * @param string $help e.g. A summary of the duration in seconds.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     * @param int $maxAgeSeconds e.g. 604800
     * @param non-empty-list<float>|null $quantiles e.g. [0.01, 0.5, 0.99]
     *
     * @throws MetricsRegistrationException
     */
    public function registerSummary(
        string $namespace,
        string $name,
        string $help,
        array $labels = [],
        int $maxAgeSeconds = 600,
        ?array $quantiles = null
    ): BetterSummary {
        $metricIdentifier = "$namespace:$name";
        if (isset($this->summaries[$metricIdentifier])) {
            throw new MetricsRegistrationException("Metric already registered");
        }
        $this->summaries[$metricIdentifier] = new BetterSummary(
            $this->storageAdapter,
            $namespace,
            $name,
            $help,
            $labels,
            $maxAgeSeconds,
            $quantiles
        );
        return $this->summaries[$metricIdentifier];
    }

    /**
     *
     * @throws MetricNotFoundException
     */
    public function getSummary(string $namespace, string $name): BetterSummary
    {
        $metricIdentifier = "$namespace:$name";
        if (!isset($this->summaries[$metricIdentifier])) {
            throw new MetricNotFoundException("Metric not found:" . $metricIdentifier);
        }
        return $this->summaries["$namespace:$name"];
    }

    /**
     * @param string $namespace e.g. cms
     * @param string $name e.g. duration_seconds
     * @param string $help e.g. A summary of the duration in seconds.
     * @param array<string, string> $labels e.g. ['controller' => 'someController', 'action' => 'someAction']
     * @param int $maxAgeSeconds e.g. 604800
     * @param non-empty-list<float>|null $quantiles e.g. [0.01, 0.5, 0.99]
     *
     * @throws MetricsRegistrationException
     */
    public function getOrRegisterSummary(
        string $namespace,
        string $name,
        string $help,
        array $labels = [],
        int $maxAgeSeconds = 600,
        ?array $quantiles = null
    ): BetterSummary {
        try {
            $summary = $this->getSummary($namespace, $name);
        } catch (MetricNotFoundException $e) {
            $summary = $this->registerSummary($namespace, $name, $help, $labels, $maxAgeSeconds, $quantiles);
        }
        return $summary;
    }
}
