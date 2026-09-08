<?php declare(strict_types=1);

namespace Amp\Pipeline\Internal;

use Amp\Cancellation;
use Amp\CancelledException;
use Amp\DeferredCancellation;
use Amp\Pipeline\ConcurrentIterator;
use Amp\Pipeline\DisposedException;
use Revolt\EventLoop;
use Revolt\EventLoop\Suspension;

/**
 * @internal
 *
 * @template-covariant T
 * @template-implements ConcurrentIterator<T>
 */
final class ConcurrentClosureIterator implements ConcurrentIterator
{
    /** @var \SplQueue<Suspension<int>> */
    private readonly \SplQueue $sources;

    /** @var QueueState<T> */
    private readonly QueueState $queue;

    private readonly Sequence $sequence;

    private readonly DeferredCancellation $deferredCancellation;

    private int $cancellations = 0;

    private int $position = 0;

    /**
     * @param \Closure(Cancellation):T $supplier
     */
    public function __construct(private readonly \Closure $supplier)
    {
        $this->sequence = $sequence = new Sequence();
        $this->queue = new QueueState();
        $this->sources = $sources = new \SplQueue();
        $this->deferredCancellation = new DeferredCancellation();

        $this->deferredCancellation->getCancellation()->subscribe(static function () use ($sources, $sequence): void {
            while (!$sources->isEmpty()) {
                $sources->dequeue();
            }

            $sequence->dispose();
        });
    }

    #[\Override]
    public function continue(?Cancellation $cancellation = null): bool
    {
        if ($this->queue->isComplete()) {
            return $this->queue->continue($cancellation);
        }

        if ($this->cancellations) {
            // A previous cancellation left a supplier callback awaiting a value, so skip enqueuing another callback.
            --$this->cancellations;
        } elseif ($this->sources->isEmpty()) {
            $queue = $this->queue;
            $sources = $this->sources;
            $sequence = $this->sequence;
            $supplier = $this->supplier;
            $deferredCancellation = $this->deferredCancellation;
            EventLoop::queue(static function (int $position) use (
                $queue,
                $sources,
                $sequence,
                $supplier,
                $deferredCancellation
            ): void {
                $suspension = EventLoop::getSuspension();

                do {
                    try {
                        $value = $supplier($deferredCancellation->getCancellation());
                    } catch (\Throwable $exception) {
                        $sequence->await($position);
                        if (!$queue->isComplete()) {
                            $queue->error($exception);
                            $deferredCancellation->cancel($exception);
                        }

                        return;
                    }

                    $sequence->await($position);

                    try {
                        if (!$queue->isComplete()) {
                            $queue->push($value);
                        }
                    } catch (DisposedException $exception) {
                        $deferredCancellation->cancel($exception);
                        return;
                    } finally {
                        $sequence->resume($position);
                    }

                    // Make this fiber available for reuse only once it is suspended below, otherwise a concurrent
                    // continue() may resume the suspension while suspended in Sequence::await() or QueueState::push().
                    $sources->enqueue($suspension);
                } while ($position = $suspension->suspend());
            }, $this->position++);
        } else {
            $suspension = $this->sources->dequeue();
            $suspension->resume($this->position++);
        }

        try {
            return $this->queue->continue($cancellation);
        } catch (CancelledException $exception) {
            // The next call to continue() will consume the pending value.
            ++$this->cancellations;
            throw $exception;
        }
    }

    #[\Override]
    public function getValue(): mixed
    {
        return $this->queue->getValue();
    }

    #[\Override]
    public function getPosition(): int
    {
        return $this->queue->getPosition();
    }

    #[\Override]
    public function isComplete(): bool
    {
        return $this->queue->isConsumed() || $this->queue->isDisposed();
    }

    #[\Override]
    public function dispose(): void
    {
        $this->queue->dispose();
        $this->deferredCancellation->cancel();
    }

    #[\Override]
    public function getIterator(): \Traversable
    {
        while ($this->continue()) {
            yield $this->getPosition() => $this->getValue();
        }
    }
}
