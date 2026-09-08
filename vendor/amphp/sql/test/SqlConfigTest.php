<?php declare(strict_types=1);

namespace Amp\Sql\Test;

use Amp\Sql\SqlConfig;
use PHPUnit\Framework\TestCase;

class SqlConfigTest extends TestCase
{
    private function createConfigFromString(string $connectionString): SqlConfig
    {
        return new class($connectionString) extends SqlConfig {
            public function __construct(string $connectionString)
            {
                $parts = self::parseConnectionString($connectionString);

                parent::__construct(
                    host: $parts["host"] ?? '',
                    port: (int) ($parts["port"] ?? 0),
                    user: $parts["user"] ?? "",
                    password: $parts["password"] ?? "",
                    database: $parts["db"] ?? "",
                );
            }
        };
    }

    public function provideValidConnectionStrings(): array
    {
        return [
            'basic' => ["host=localhost port=5432 user=user pass=test db=test"],
            'alternative' => ["host=localhost;port=5432;user=user;password=test;db=test"],
            'port-in-host' => ["host=localhost:5432 user=user pass=test db=test"],
            'whitespace' => ["   host=localhost   port=5432   user=user   pass=test   db=test   "],
            'whitespace-after-semicolon' => ["host=localhost; port=5432; user=user; password=test; db=test; "],
            'quotes' => ['host="localhost" port=5432 user="user" pass="test" db="test"'],
            'alternative-with-whitespace' => ["host=localhost; port=5432; user=user; password=test; db=test;"],
        ];
    }

    /**
     * @dataProvider provideValidConnectionStrings
     */
    public function testValidStrings(string $connectionString): void
    {
        $config = $this->createConfigFromString($connectionString);

        self::assertSame("localhost", $config->getHost());
        self::assertSame(5432, $config->getPort());
        self::assertSame("user", $config->getUser());
        self::assertSame("test", $config->getPassword());
        self::assertSame("test", $config->getDatabase());
    }

    public function testQuoteInQuotedValue(): void
    {
        $config = $this->createConfigFromString(
            <<<'CS'
            host="local\"host:3306" database='test'
            CS
        );

        self::assertSame('local"host', $config->getHost());
        self::assertSame(3306, $config->getPort());
        self::assertSame('test', $config->getDatabase());
    }

    public function testEscapedSpace(): void
    {
        $config = $this->createConfigFromString(
            <<<'CS'
            host=localhost user=Test\ User database=Test\ Name
            CS
        );

        self::assertSame('localhost', $config->getHost());
        self::assertSame('Test User', $config->getUser());
        self::assertSame('Test Name', $config->getDatabase());
    }

    public function provideInvalidConnectionStrings(): array
    {
        return [
            'missing-value' => ['host='],
            'empty-value' => ['host=""'],
            'leading-characters' => ['test host=localhost'],
            'trailing-characters' => ['host=localhost test'],
            'invalid-whitespace' => ['host= localhost'],
            'duplicate-key' => ['host=localhost port=5432 port=5433']
        ];
    }

    /**
     * @dataProvider provideInvalidConnectionStrings
     */
    public function testInvalidStrings(string $connectionString): void
    {
        $this->expectException(\ValueError::class);
        $this->createConfigFromString($connectionString);
    }
}
