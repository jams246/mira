<?php

declare(strict_types=1);

namespace App\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;

class IPAddressValueObject extends StringValueObject
{
    private const ALLOWED_VALUE = 'Unknown';

    public function __construct(string $value)
    {
        if (!\strcmp(self::ALLOWED_VALUE, $value) xor !\filter_var($value, FILTER_VALIDATE_IP)) {
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
