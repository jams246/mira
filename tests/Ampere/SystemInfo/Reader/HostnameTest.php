<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\Hostname;
use PHPUnit\Framework\TestCase;

class HostnameTest extends TestCase
{
    /**
     * @dataProvider validStringValueProvider
     */
    public function testParseRemoveLineBreakAndNewLine(string $input, string $expected): void
    {
        $object = new Hostname();

        $response = $object->parse($input);

        $this->assertSame($expected, $response);
    }

    public function validStringValueProvider(): array
    {
        return [
            [
                "Test\n",
                'Test',
            ],
            [
                "Test\r",
                'Test',
            ],
            [
                "\rTest\n",
                'Test',
            ],
        ];
    }
}
