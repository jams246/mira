<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class StringValueObject
{
    public function __construct(private string $value)
    {
        if (0 === \strlen($this->value)) {
            throw new ValueObjectException('String length cannot be zero.');
        }
    }

    public function getValue(): string
    {
        return $this->value;
    }
}
