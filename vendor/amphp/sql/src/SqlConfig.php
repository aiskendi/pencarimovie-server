<?php declare(strict_types=1);

namespace Amp\Sql;

abstract class SqlConfig
{
    public const KEY_MAP = [
        'hostname' => 'host',
        'username' => 'user',
        'pass' => 'password',
        'database' => 'db',
        'dbname' => 'db',
    ];

    private const KEY_VALUE_PAIR_REGEXP = <<<'REGEXP'
        [\G\s*(\w+)=((['"])(?:\\(?:\\|\3)|(?!\3).)*+\3|[^ '";](?:\\(?:\\| )|(?!\s+|;| ).)*+)(?:\s+|;|$)]
        REGEXP;

    private string $host;

    private int $port;

    private ?string $user;

    private ?string $password;

    private ?string $database;

    /**
     * Parses a connection string into an array of keys and values given.
     *
     * @param string $connectionString Connection string, e.g., "hostname=localhost username=sql password=default"
     * @param array<non-empty-string, non-empty-string> $keymap Map of alternative key names to canonical key names.
     *
     * @return array<non-empty-string, string>
     */
    protected static function parseConnectionString(string $connectionString, array $keymap = self::KEY_MAP): array
    {
        $values = [];
        $connectionString = \trim($connectionString);

        if ($connectionString === '') {
            throw new \ValueError("Empty connection string");
        }

        if (\preg_match_all(
            pattern: self::KEY_VALUE_PAIR_REGEXP,
            subject: $connectionString,
            matches: $matches,
            flags: \PREG_SET_ORDER | \PREG_UNMATCHED_AS_NULL,
        ) === false) {
            throw new \ValueError("Invalid connection string");
        }

        $offset = 0;
        foreach ($matches as [$pair, $key, $value, $quote]) {
            \assert($value !== null);

            if ($quote !== null) {
                $value = \stripslashes(\substr($value, 1, -1));

                if ($value === '') {
                    throw new \ValueError("Empty connection string value for key '{$key}'");
                }
            } else {
                $value = \str_replace('\\ ', ' ', $value);
            }

            \assert($key !== null && $key !== '');
            $key = $keymap[$key] ?? $key;
            if (\array_key_exists($key, $values)) {
                throw new \ValueError("Duplicate connection string key '{$key}'");
            }

            $values[$key] = $value;

            \assert($pair !== null);
            $offset += \strlen($pair);
        }

        if ($offset !== \strlen($connectionString)) {
            throw new \ValueError("Trailing characters in connection string");
        }

        if (\preg_match('[^(?<host>.+):(?<port>\d{1,5})$]', $values["host"] ?? "", $match)) {
            $values["host"] = $match["host"];
            $values["port"] = $match["port"];
        }

        return $values;
    }

    public function __construct(
        string $host,
        int $port,
        ?string $user = null,
        ?string $password = null,
        ?string $database = null
    ) {
        $this->host = $host;
        $this->port = $port;
        $this->user = $user;
        $this->password = $password;
        $this->database = $database;
    }

    final public function getHost(): string
    {
        return $this->host;
    }

    final public function withHost(string $host): static
    {
        $new = clone $this;
        $new->host = $host;
        return $new;
    }

    final public function getPort(): int
    {
        return $this->port;
    }

    final public function withPort(int $port): static
    {
        $new = clone $this;
        $new->port = $port;
        return $new;
    }

    final public function getUser(): ?string
    {
        return $this->user;
    }

    final public function withUser(?string $user = null): static
    {
        $new = clone $this;
        $new->user = $user;
        return $new;
    }

    final public function getPassword(): ?string
    {
        return $this->password;
    }

    final public function withPassword(?string $password = null): static
    {
        $new = clone $this;
        $new->password = $password;
        return $new;
    }

    final public function getDatabase(): ?string
    {
        return $this->database;
    }

    final public function withDatabase(?string $database = null): static
    {
        $new = clone $this;
        $new->database = $database;
        return $new;
    }
}
