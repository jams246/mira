<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class BitVersionValueObject extends StringValueObject
{
    private const ALLOWED_LIST = ['Unknown', '32bit', '64bit'];

    public function __construct(string $value)
    {
        if (!\in_array($value, self::ALLOWED_LIST)) {
            $errorMessage = \sprintf(
                '%s only accepts allowed values. Input was %s',
                __CLASS__,
                $value
            );
            throw new ValueObjectException($errorMessage);
        }

        parent::__construct($value);
    }
}
