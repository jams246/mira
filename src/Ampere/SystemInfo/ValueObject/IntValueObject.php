<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

class IntValueObject
{
    public function __construct(private int $value)
    {
    }

    public function getValue(): int
    {
        return $this->value;
    }
}
