Feature: Buy things with apple

    Scenario: Buy fruits
        Given A wallet with 50

        When I buy some apples for 1
        And I buy some bananas for 2

        Then I have 47 left
