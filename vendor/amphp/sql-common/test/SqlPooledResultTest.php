<?php declare(strict_types=1);

namespace Amp\Sql\Common\Test;

use Amp\PHPUnit\AsyncTestCase;
use Amp\Sql\Common\Test\Stub\StubSqlPooledResult;
use Amp\Sql\Common\Test\Stub\StubSqlResult;
use function Amp\delay;

class SqlPooledResultTest extends AsyncTestCase
{
    public function testIdleConnectionsRemovedAfterTimeout()
    {
        $invoked = false;

        $release = function () use (&$invoked) {
            $invoked = true;
        };

        $expectedRow = ['column' => 'value'];

        $secondResult = new StubSqlResult([$expectedRow]);
        $firstResult = new StubSqlResult([$expectedRow], $secondResult);
        $pooledResult = new StubSqlPooledResult(new StubSqlResult([$expectedRow], $firstResult), $release);

        $iterator = $pooledResult->getIterator();

        self::assertSame($expectedRow, $iterator->current());

        self::assertFalse($invoked);

        $iterator->next();
        self::assertFalse($iterator->valid());

        self::assertFalse($invoked); // Next result set available.

        $pooledResult = $pooledResult->getNextResult();
        $iterator = $pooledResult->getIterator();

        self::assertSame($expectedRow, $iterator->current());

        $iterator->next();
        self::assertFalse($iterator->valid());

        $pooledResult = $pooledResult->getNextResult();
        unset($pooledResult); // Manually unset to trigger destructor.

        delay(0); // Tick event loop to dispose of result set.

        self::assertTrue($invoked); // No next result set, so release callback invoked.
    }

    public function testIteratorRetainsReference(): void
    {
        $expectedRow = ['column' => 'value'];
        $expectedRows = [$expectedRow, $expectedRow, $expectedRow];
        $stubResult = new StubSqlResult($expectedRows);

        $invoked = false;
        $release = function () use (&$invoked) {
            $invoked = true;
        };

        $iterationCount = 0;
        foreach ((new StubSqlPooledResult($stubResult, $release)) as $row) {
            ++$iterationCount;

            delay(0); // Tick event loop to allow entry into disposal function if queued in event loop.

            self::assertSame($expectedRow, $row);
            self::assertFalse($invoked);
        }

        self::assertSame(\count($expectedRows), $iterationCount);

        delay(0); // Tick event loop to dispose of result set.

        self::assertTrue($invoked);
    }
}
