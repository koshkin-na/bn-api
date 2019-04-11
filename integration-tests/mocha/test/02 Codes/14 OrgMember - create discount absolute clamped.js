const supertest = require('supertest');
const expect = require('chai').expect;
const mocha = require('mocha');
const tv4 = require('tv4');
const fs = require('fs');
const pm = require('../pm')

const baseUrl = supertest(pm.environment.get('server'));

const apiEndPoint = '/events/{{last_event_id}}/codes';


var response;
var responseBody;


const post = async function (request_body) {
    return baseUrl
        .post(pm.substitute(apiEndPoint))
        .set('Accept', 'application/json')
        .set('Content-Type', 'application/json')
        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .send(pm.substitute(request_body));
};

const get = async function (request_body) {
    return baseUrl
        .get(pm.substitute(apiEndPoint))

        .set('Authorization', pm.substitute('Bearer {{org_member_token}}'))

        .set('Accept', 'application/json')
        .send();
};

let requestBody = `{
	"name":"Discount Tickets",
	"code_type" : "Discount",
	"redemption_codes" : ["RedeemDiscountCode{{$timestamp}}_absolute_clamped"],
	"max_uses" : 10,
	"discount_in_cents" : 9999,
	"start_date": "2018-01-01T12:00:00",
	"end_date": "2059-01-01T12:00:00",
	"max_tickets_per_user" : 2,
	"ticket_type_ids": ["{{last_ticket_type_id}}"]
}`;


describe('OrgMember - create discount absolute clamped', function () {
    before(async function () {
        response = await post(requestBody);
        console.log(response.request.header);
        console.log(response.request.url);
        console.log(response.request._data);
        console.log(response.request.method);
        responseBody = JSON.stringify(response.body);
        //console.log(pm);
        console.log(response.status);
        console.log(responseBody);
    });

    after(async function () {
        // add after methods


    });

    it("should be 201", function () {
        expect(response.status).to.equal(201);
    });


    it("discount should have correct information", function () {

        let json = JSON.parse(responseBody);
        pm.environment.set("last_code_id", json.id);
        pm.environment.set("discount_redemption_code_clamped", json.redemption_codes[0]);
        expect(json.name).to.equal("Discount Tickets");

        expect(json.max_uses).to.equal(10);
        expect(json.code_type).to.equal("Discount");
        expect(json.discount_in_cents).to.equal(9999);
        expect(json.start_date).to.equal("2018-01-01T12:00:00");
        expect(json.end_date).to.equal("2059-01-01T12:00:00");
        expect(json.max_tickets_per_user).to.equal(2);
        let ticket_type_id = pm.variables.get("last_ticket_type_id");
        expect(json.ticket_type_ids[0]).to.equal(ticket_type_id);
    });


});

            
